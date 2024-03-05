use crate::{syscall::errno::*, task::current_task, timer::TimeSpec};
use alloc::{collections::BTreeMap, sync::Arc};
use log::*;
use num_enum::FromPrimitive;

use super::{
    block_current_and_run_next,
    manager::{wait_with_timeout, WaitQueue},
};

#[allow(unused)]
#[derive(Debug, Eq, PartialEq, FromPrimitive)]
#[repr(u32)]
pub enum FutexCmd {
    /// This  operation  tests  that  the value at the futex
    /// word pointed to by the address uaddr still  contains
    /// the expected value val, and if so, then sleeps wait‐
    /// ing for a FUTEX_WAKE operation on  the  futex  word.
    /// The load of the value of the futex word is an atomic
    /// memory access (i.e., using atomic  machine  instruc‐
    /// tions  of  the respective architecture).  This load,
    /// the comparison with the expected value, and starting
    /// to  sleep  are  performed atomically and totally or‐
    /// dered with respect to other futex operations on  the
    /// same  futex word.  If the thread starts to sleep, it
    /// is considered a waiter on this futex word.   If  the
    /// futex  value does not match val, then the call fails
    /// immediately with the error EAGAIN.
    Wait = 0,
    /// This operation wakes at most val of the waiters that
    /// are waiting (e.g., inside FUTEX_WAIT) on  the  futex
    /// word  at  the  address uaddr.  Most commonly, val is
    /// specified as either 1 (wake up a single  waiter)  or
    /// INT_MAX (wake up all waiters).  No guarantee is pro‐
    /// vided about which waiters are awoken (e.g., a waiter
    /// with  a higher scheduling priority is not guaranteed
    /// to be awoken in preference to a waiter with a  lower
    /// priority).
    Wake = 1,
    Fd = 2,
    Requeue = 3,
    CmpRequeue = 4,
    WakeOp = 5,
    LockPi = 6,
    UnlockPi = 7,
    TrylockPi = 8,
    WaitBitset = 9,
    #[num_enum(default)]
    Invalid,
}

pub struct Futex {
    inner: BTreeMap<usize, WaitQueue>,
}

/// Currently the `rt_clk` is ignored.
pub fn do_futex_wait(futex_word: &mut u32, val: u32, timeout: Option<TimeSpec>) -> isize {
    let timeout = timeout.map(|t| t + TimeSpec::now());
    let futex_word_addr = futex_word as *const u32 as usize;
    if *futex_word != val {
        trace!(
            "[futex] --wait-- **not match** futex: {:X}, val: {:X}",
            *futex_word,
            val
        );
        return EAGAIN;
    } else {
        let task = current_task().unwrap();
        let mut futex = task.futex.lock();
        let mut wait_queue = if let Some(wait_queue) = futex.inner.remove(&futex_word_addr) {
            wait_queue
        } else {
            WaitQueue::new()
        };
        // push to wait queue
        wait_queue.add_task(Arc::downgrade(&task));
        futex.inner.insert(futex_word_addr, wait_queue);
        // if has a timeout, also push to timeout waitqueue
        if let Some(timeout) = timeout {
            trace!("[do_futex_wait] sleep with timeout: {:?}", timeout);
            wait_with_timeout(Arc::downgrade(&task), timeout);
        }
        drop(futex);
        drop(task);

        block_current_and_run_next();
        let task = current_task().unwrap();
        let inner = task.acquire_inner_lock();
        // woke by signal
        if !inner.sigpending.difference(inner.sigmask).is_empty() {
            return EINTR;
        }
        SUCCESS
    }
}

impl Futex {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
    pub fn wake(&mut self, futex_word_addr: usize, val: u32) -> isize {
        if let Some(mut wait_queue) = self.inner.remove(&futex_word_addr) {
            let ret = wait_queue.wake_at_most(val as usize);
            if !wait_queue.is_empty() {
                self.inner.insert(futex_word_addr, wait_queue);
            }
            ret as isize
        } else {
            0
        }
    }
    pub fn requeue(&mut self, futex_word: &u32, futex_word_2: &u32, val: u32, val2: u32) -> isize {
        let futex_word_addr = futex_word as *const u32 as usize;
        let futex_word_addr_2 = futex_word_2 as *const u32 as usize;
        let wake_cnt = if val != 0 {
            self.wake(futex_word_addr, val)
        } else {
            0
        };
        if let Some(mut wait_queue) = self.inner.remove(&futex_word_addr) {
            let mut wait_queue_2 = if let Some(wait_queue) = self.inner.remove(&futex_word_addr_2) {
                wait_queue
            } else {
                WaitQueue::new()
            };
            let mut requeue_cnt = 0;
            if val2 != 0 {
                while let Some(task) = wait_queue.pop_task() {
                    wait_queue_2.add_task(task);
                    requeue_cnt += 1;
                    if requeue_cnt == val2 as isize {
                        break;
                    }
                }
            }
            if !wait_queue.is_empty() {
                self.inner.insert(futex_word_addr, wait_queue);
            }
            if !wait_queue_2.is_empty() {
                self.inner.insert(futex_word_addr_2, wait_queue_2);
            }
            wake_cnt + requeue_cnt
        } else {
            wake_cnt
        }
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
