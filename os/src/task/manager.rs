use core::cmp::Ordering;

#[cfg(feature = "oom_handler")]
use crate::config::SYSTEM_TASK_LIMIT;
#[cfg(feature = "oom_handler")]
use alloc::vec::Vec;

use crate::timer::TimeSpec;

use super::{current_task, TaskControlBlock};
use alloc::collections::{BinaryHeap, VecDeque};
use alloc::sync::{Arc, Weak};
use lazy_static::*;
use spin::Mutex;

#[cfg(feature = "oom_handler")]
pub struct ActiveTracker {
    bitmap: Vec<u64>,
}

#[cfg(feature = "oom_handler")]
#[allow(unused)]
impl ActiveTracker {
    pub const DEFAULT_SIZE: usize = SYSTEM_TASK_LIMIT;
    pub fn new() -> Self {
        let len = (Self::DEFAULT_SIZE + 63) / 64;
        let mut bitmap = Vec::with_capacity(len);
        bitmap.resize(len, 0);
        Self { bitmap }
    }
    pub fn check_active(&self, pid: usize) -> bool {
        (self.bitmap[pid / 64] & (1 << (pid % 64))) != 0
    }
    pub fn check_inactive(&self, pid: usize) -> bool {
        (self.bitmap[pid / 64] & (1 << (pid % 64))) == 0
    }
    pub fn mark_active(&mut self, pid: usize) {
        self.bitmap[pid / 64] |= 1 << (pid % 64)
    }
    pub fn mark_inactive(&mut self, pid: usize) {
        self.bitmap[pid / 64] &= !(1 << (pid % 64))
    }
}

#[cfg(feature = "oom_handler")]
pub struct TaskManager {
    pub ready_queue: VecDeque<Arc<TaskControlBlock>>,
    pub interruptible_queue: VecDeque<Arc<TaskControlBlock>>,
    pub active_tracker: ActiveTracker,
}

#[cfg(not(feature = "oom_handler"))]
pub struct TaskManager {
    pub ready_queue: VecDeque<Arc<TaskControlBlock>>,
    pub interruptible_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    #[cfg(feature = "oom_handler")]
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            interruptible_queue: VecDeque::new(),
            active_tracker: ActiveTracker::new(),
        }
    }
    #[cfg(not(feature = "oom_handler"))]
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            interruptible_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    #[cfg(feature = "oom_handler")]
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        match self.ready_queue.pop_front() {
            Some(task) => {
                self.active_tracker.mark_active(task.pid.0);
                Some(task)
            }
            None => None,
        }
    }
    #[cfg(not(feature = "oom_handler"))]
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
    pub fn add_interruptible(&mut self, task: Arc<TaskControlBlock>) {
        self.interruptible_queue.push_back(task);
    }
    pub fn drop_interruptible(&mut self, task: &Arc<TaskControlBlock>) {
        self.interruptible_queue
            .retain(|task_in_queue| Arc::as_ptr(task_in_queue) != Arc::as_ptr(task));
    }
    pub fn find_by_pid(&self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue
            .iter()
            .chain(self.interruptible_queue.iter())
            .find(|task| task.pid.0 == pid)
            .cloned()
    }
    pub fn find_by_tgid(&self, tgid: usize) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue
            .iter()
            .chain(self.interruptible_queue.iter())
            .find(|task| task.tgid == tgid)
            .cloned()
    }
    pub fn ready_count(&self) -> u16 {
        self.ready_queue.len() as u16
    }
    pub fn interruptible_count(&self) -> u16 {
        self.interruptible_queue.len() as u16
    }
    /// This function will drop `task` from `interruptible_queue` and push it into `ready_queue`.
    /// The `task` will be scheduled if everything goes well. Do nothing if `task` is already waken.
    /// # Attention
    /// This function **won't** change `task_status`, you should change it manully to keep consistency.
    pub fn wake_interruptible(&mut self, task: Arc<TaskControlBlock>) {
        match self.try_wake_interruptible(task) {
            Ok(_) => {}
            Err(_) => {
                log::trace!("[wake_interruptible] already waken");
            }
        }
    }
    /// This function will drop `task` from `interruptible_queue` and push it into `ready_queue`.
    /// The `task` will be scheduled if everything goes well. Returns `Err()` if `task` is already waken.
    /// # Attention
    /// This function **won't** change `task_status`, you should change it manully to keep consistency.
    pub fn try_wake_interruptible(
        &mut self,
        task: Arc<TaskControlBlock>,
    ) -> Result<(), WaitQueueError> {
        self.drop_interruptible(&task);
        if self.find_by_pid(task.pid.0).is_none() {
            self.add(task);
            Ok(())
        } else {
            Err(WaitQueueError::AlreadyWaken)
        }
    }
    #[allow(unused)]
    // debug use only
    pub fn show_ready(&self) {
        self.ready_queue.iter().for_each(|task| {
            log::error!("[show_ready] pid: {}", task.pid.0);
        })
    }
    #[allow(unused)]
    // debug use only
    pub fn show_interruptible(&self) {
        self.interruptible_queue.iter().for_each(|task| {
            log::error!("[show_interruptible] pid: {}", task.pid.0);
        })
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().fetch()
}

/// Try to clean all tasks' memory space until `req` pages are released.
#[cfg(feature = "oom_handler")]
pub fn do_oom(req: usize) -> Result<(), ()> {
    let mut manager = TASK_MANAGER.lock();
    let mut cleaned = Vec::with_capacity(16);
    let mut total_released = 0;
    for task in manager
        .interruptible_queue
        .iter()
        .filter(|task| manager.active_tracker.check_active(task.pid.0))
    {
        let released = task.vm.lock().do_deep_clean();
        log::warn!("deep clean on task: {}, released: {}", task.tgid, released);
        cleaned.push(task.pid.0);
        total_released += released;
        if total_released >= req {
            while let Some(pid) = cleaned.pop() {
                manager.active_tracker.mark_inactive(pid)
            }
            return Ok(());
        };
    }
    for task in manager
        .ready_queue
        .iter()
        .rev()
        .filter(|task| manager.active_tracker.check_active(task.pid.0))
    {
        let released = task.vm.lock().do_shallow_clean();
        log::warn!(
            "shallow clean on task: {}, released: {}",
            task.tgid,
            released
        );
        cleaned.push(task.pid.0);
        total_released += released;
        if total_released >= req {
            while let Some(pid) = cleaned.pop() {
                manager.active_tracker.mark_inactive(pid)
            }
            return Ok(());
        };
    }
    Err(())
}

#[cfg(not(feature = "oom_handler"))]
#[allow(unused)]
pub fn do_oom() {
    // do nothing
}

/// This function add a `task` to `interruptible_queue`,
/// but won't take it out from `ready_queue`.
/// So you should make sure that the `task` won't be presented in `ready_queue`.
/// In common cases, a `task` will be dropped from `ready_queue` when it is scheduled,
/// and you can use `take_current_task()` to acquire the ownership of current `task`.
/// # Attention
/// You should find a place to save `Arc<TaskControlBlock>` of the task, or you would
/// be unable to use `wake_interruptible()` to wake it up in the future.
/// This function **won't** change `task_status`, you should change it manully to keep consistency.
pub fn sleep_interruptible(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add_interruptible(task);
}

/// This function will drop `task` from `interruptible_queue` and push it into `ready_queue`.
/// The `task` will be scheduled if everything goes well. Do nothing if `task` is already waken.
/// # Attention
/// This function **won't** change `task_status`, you should change it manully to keep consistency.
pub fn wake_interruptible(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().wake_interruptible(task)
}

/// # Warning
/// `pid` here is unique, user will regard it as `tid`
pub fn find_task_by_pid(pid: usize) -> Option<Arc<TaskControlBlock>> {
    let task = current_task().unwrap();
    if task.pid.0 == pid {
        Some(task)
    } else {
        TASK_MANAGER.lock().find_by_pid(pid)
    }
}

/// Return arbitrary task with `tgid`.
pub fn find_task_by_tgid(tgid: usize) -> Option<Arc<TaskControlBlock>> {
    let task = current_task().unwrap();
    if task.tgid == tgid {
        Some(task)
    } else {
        TASK_MANAGER.lock().find_by_tgid(tgid)
    }
}

pub fn procs_count() -> u16 {
    let manager = TASK_MANAGER.lock();
    manager.ready_count() + manager.interruptible_count()
}

pub enum WaitQueueError {
    AlreadyWaken,
}

pub struct WaitQueue {
    inner: VecDeque<Weak<TaskControlBlock>>,
}

#[allow(unused)]
impl WaitQueue {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }
    /// This function add a `task` to `WaitQueue` but **won't** block it,
    /// if you want to block a `task`, use `block_current_and_run_next()`.
    pub fn add_task(&mut self, task: Weak<TaskControlBlock>) {
        self.inner.push_back(task);
    }
    /// This function will try to pop a `task` from `WaitQueue` but **won't** wake it up
    pub fn pop_task(&mut self) -> Option<Weak<TaskControlBlock>> {
        self.inner.pop_front()
    }
    /// Returns `true` if the `WaitQueue` contains an element equal to the given `task`
    pub fn contains(&self, task: &Weak<TaskControlBlock>) -> bool {
        self.inner
            .iter()
            .any(|task_in_queue| Weak::as_ptr(task_in_queue) == Weak::as_ptr(task))
    }
    /// Returns `true` if the `WaitQueue` is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// This funtion will wake up all `task` in this `WaitQueue` and change their `task_status`
    /// to `Ready`, These `task` will be scheduled in future if everything goes well.
    /// # Warning
    /// This function will call `acquire_inner_lock` for each `task` in `WaitQueue`, so be careful of **deadlock**.
    pub fn wake_all(&mut self) -> usize {
        self.wake_at_most(usize::MAX)
    }
    /// Wake up a number of `task` no more than `limit` in this `WaitQueue`. Returns the number of
    /// waken `task`.
    /// # Warning
    /// This function will call `acquire_inner_lock` for each waken `task`, so be careful of **deadlock**.
    pub fn wake_at_most(&mut self, limit: usize) -> usize {
        if limit == 0 {
            return 0;
        }
        let mut manager = TASK_MANAGER.lock();
        let mut cnt = 0;
        while let Some(task) = self.inner.pop_front() {
            match task.upgrade() {
                Some(task) => {
                    let mut inner = task.acquire_inner_lock();
                    match inner.task_status {
                        super::TaskStatus::Interruptible => {
                            inner.task_status = super::task::TaskStatus::Ready
                        }
                        // for `Ready` or `Running`, we don't need to do wake,
                        // for `Zombie`, we will mess up the process management if we do wake...
                        _ => continue,
                    }
                    drop(inner);
                    if manager.try_wake_interruptible(task).is_ok() {
                        cnt += 1;
                    }
                    if cnt == limit {
                        break;
                    }
                }
                // task is dead, just ignore
                None => continue,
            }
        }
        cnt
    }
}

pub struct TimeoutWaiter {
    task: Weak<TaskControlBlock>,
    timeout: TimeSpec,
}

// BinaryHeap is max-heap, so we need to reverse the ord
impl Ord for TimeoutWaiter {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::reverse(self.timeout.cmp(&other.timeout))
    }
}

impl PartialOrd for TimeoutWaiter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TimeoutWaiter {}

impl PartialEq for TimeoutWaiter {
    fn eq(&self, other: &Self) -> bool {
        self.timeout.eq(&other.timeout)
    }
}

pub struct TimeoutWaitQueue {
    inner: BinaryHeap<TimeoutWaiter>,
}

impl TimeoutWaitQueue {
    pub fn new() -> Self {
        Self {
            inner: BinaryHeap::new(),
        }
    }
    /// This function add a `task` to `WaitQueue` but **won't** block it,
    /// if you want to block a `task`, use `block_current_and_run_next()`.
    pub fn add_task(&mut self, task: Weak<TaskControlBlock>, timeout: TimeSpec) {
        self.inner.push(TimeoutWaiter { task, timeout });
    }
    pub fn wake_expired(&mut self, now: TimeSpec) {
        let mut manager = TASK_MANAGER.lock();
        while let Some(waiter) = self.inner.pop() {
            // the remaining tasks in heap haven't reach their timeout
            if waiter.timeout > now {
                log::trace!(
                    "[wake_expired] no more expired, next pending task timeout: {:?}, now: {:?}",
                    waiter.timeout,
                    now
                );
                self.inner.push(waiter);
                break;
            // wake one task
            } else {
                match waiter.task.upgrade() {
                    Some(task) => {
                        let mut inner = task.acquire_inner_lock();
                        match inner.task_status {
                            super::TaskStatus::Interruptible => {
                                inner.task_status = super::task::TaskStatus::Ready
                            }
                            // for `Ready` or `Running`, we don't need to do wake,
                            // for `Zombie`, we will mess up the process management if we do wake...
                            _ => continue,
                        }
                        drop(inner);
                        log::trace!(
                            "[wake_expired] pid: {}, timeout: {:?}",
                            task.pid.0,
                            waiter.timeout
                        );
                        manager.wake_interruptible(task);
                    }
                    // task is dead, just ignore
                    None => continue,
                }
            }
        }
    }
    #[allow(unused)]
    // debug use only
    pub fn show_waiter(&self) {
        for waiter in self.inner.iter() {
            log::error!("[show_waiter] timeout: {:?}", waiter.timeout);
        }
    }
}

lazy_static! {
    pub static ref TIMEOUT_WAITQUEUE: Mutex<TimeoutWaitQueue> = Mutex::new(TimeoutWaitQueue::new());
}

/// This function add a `task` to `TIMEOUT_WAITQUEUE` but won't block it,
/// if you want to block a `task`, use `block_current_and_run_next()`
pub fn wait_with_timeout(task: Weak<TaskControlBlock>, timeout: TimeSpec) {
    TIMEOUT_WAITQUEUE.lock().add_task(task, timeout)
}

/// Wake all expired waiter on `TIMEOUT_WAITQUEUE`
pub fn do_wake_expired() {
    TIMEOUT_WAITQUEUE
        .lock()
        .wake_expired(crate::timer::TimeSpec::now());
}
