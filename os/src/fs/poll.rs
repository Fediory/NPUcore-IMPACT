use crate::{
    mm::try_get_from_user, syscall::errno::EFAULT, task::signal::Signals, timer::TimeSpec,
};
use alloc::vec::Vec;
use core::ptr::null_mut;

use crate::{
    mm::{copy_from_user_array, copy_to_user_array},
    task::{current_task, sigprocmask, suspend_current_and_run_next, SigMaskHow},
};

///  A scheduling  scheme  whereby  the  local  process  periodically  checks  until  the  pre-specified events (for example, read, write) have occurred.
/// The PollFd struct in 32-bit style.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    /// File descriptor
    fd: u32,
    /// Requested events
    events: PollEvent,
    /// Returned events
    revents: PollEvent,
}

bitflags! {
    /// Event types that can be polled for.
    ///
    /// These bits may be set in `events`(see `ppoll()`) to indicate the interesting event types;
    ///
    /// they will appear in `revents` to indicate the status of the file descriptor.
    struct PollEvent:u16 {
    /// There is data to read.
    const POLLIN = 0x001;
    /// There is urgent data to read.
    const POLLPRI = 0x002;
    /// Writing now will not block.
    const POLLOUT = 0x004;

    // These values are defined in XPG4.2.
    /// Normal data may be read.
    const POLLRDNORM = 0x040;
    /// Priority data may be read.
    const POLLRDBAND = 0x080;
    /// Writing now will not block.
    const POLLWRNORM = 0x100;
    /// Priority data may be written.
    const POLLWRBAND = 0x200;


    /// Linux Extension.
    const POLLMSG = 0x400;
    /// Linux Extension.
    const POLLREMOVE = 0x1000;
    /// Linux Extension.
    const POLLRDHUP = 0x2000;

    /* Event types always implicitly polled for.
    These bits need not be set in `events',
    but they will appear in `revents' to indicate the status of the file descriptor.*/

    /// Implicitly polled for only.
    /// Error condition.
    const POLLERR = 0x008;
    /// Implicitly polled for only.
    /// Hung up.
    const POLLHUP = 0x010;
    /// Implicitly polled for only.
    /// Invalid polling request.
    const POLLNVAL = 0x020;
    }
}

/// Wait for one of the events in `poll_fd_p` to happen, or the time limit to run out if any.
/// Unlike the function family of `select()` which are basically AND'S,
/// `poll()`'s act like OR's for polling the files.
/// # Arguments
/// * `poll_fd`: The USER pointer to the array of file descriptors to be polled
/// * `nfds`: The number stored in the previous array.
/// * `time_spec`: The time, see `timer::TimeSpec` for information. NOT SUPPORTED and will be ignored!
/// * `sigmask`: The pointer to the sigmask in use during the poll.
/// # Note
/// * `POLLHUP`, `POLLNVAL` and `POLLERR` are ALWAYS polled for all given files,
///   regardless of whether it is set in the array.
/// # Unsupported Features
/// * Timeout is not yet supported.
/// * Other implementations are supported by specific files and may not be used by
/// * Currently only user space structs are supported.
/// # Return Conditions
/// The call will block until either:
/// * a file descriptor becomes ready;
/// * the call is interrupted by a signal handler; or
/// * the timeout expires.
/// # Return Values and Side-effects
/// * On success, a positive number is returned; this is the number of structures
///   which have nonzero revents fields (in other words, those descriptors
///   with events or errors reported).
/// * A value of 0 indicates that the call timed out and no file descriptors were ready.
/// * On error, -1 is returned, and errno is set appropriately.
/// * The observed event is written back to the array, with others cleared.
pub fn ppoll(
    fds: *mut PollFd,
    nfds: usize,
    tmo_p: *const TimeSpec,
    sigmask: *const Signals,
) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let timeout: Option<TimeSpec> = match try_get_from_user(token, tmo_p) {
        Ok(tmo) => match tmo {
            Some(tmo) => Some(tmo + crate::timer::TimeSpec::now()),
            None => None,
        },
        Err(errno) => return errno,
    };
    // push to the top of TrapContext page, make use of redundant space
    let oldsig =
        ((task.trap_cx_user_va() + crate::config::PAGE_SIZE) as *mut Signals).wrapping_sub(1);
    if !sigmask.is_null() {
        sigprocmask(SigMaskHow::SIG_SETMASK.bits(), sigmask, oldsig);
    }
    drop(task);

    let mut poll_fd = Vec::<PollFd>::with_capacity(nfds);
    let mut done: isize = 0;
    if let Ok(_) = copy_from_user_array(token, fds, poll_fd.as_mut_ptr(), nfds) {
        unsafe {
            poll_fd.set_len(nfds);
        }
        for poll_fd in poll_fd.iter_mut() {
            poll_fd.revents = PollEvent::empty();
        }

        loop {
            let task = current_task().unwrap();
            let fd_table = task.files.lock();

            for poll_fd in poll_fd.iter_mut() {
                let fd = poll_fd.fd as usize;
                match fd_table.get_ref(fd) {
                    Ok(file_descriptor) => {
                        let mut trigger = 0;
                        if file_descriptor.file.hang_up() {
                            poll_fd.revents |= PollEvent::POLLHUP;
                            trigger = 1;
                        }
                        if poll_fd.events.contains(PollEvent::POLLIN)
                            && file_descriptor.file.r_ready()
                        {
                            poll_fd.revents |= PollEvent::POLLIN;
                            trigger = 1;
                        }
                        if poll_fd.events.contains(PollEvent::POLLOUT)
                            && file_descriptor.file.w_ready()
                        {
                            poll_fd.revents |= PollEvent::POLLOUT;
                            trigger = 1;
                        }
                        done += trigger;
                    }
                    Err(_) => continue,
                }
            }
            if done > 0 {
                break;
            }
            if let Some(timeout) = timeout {
                if crate::timer::TimeSpec::now() >= timeout {
                    break;
                }
            }
            drop(fd_table);
            drop(task);
            suspend_current_and_run_next();
        }

        log::trace!("[ppoll] result: {:?}", poll_fd);
        copy_to_user_array(token, &poll_fd[0], fds, nfds).unwrap();
    } else {
        log::error!(
            "[ppoll] Error copy_from_user_array(_, fds: {:?}, poll_fd.as_mut_ptr():{:?}, _)",
            fds,
            poll_fd.as_mut_ptr()
        );
        done = EFAULT;
    }
    if !sigmask.is_null() {
        sigprocmask(
            SigMaskHow::SIG_SETMASK.bits(),
            oldsig,
            null_mut::<Signals>(),
        );
    }
    done
}

// This may be unsafe since the size of bits is undefined.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// Bitmap used by `pselect()` and `select` to indicate the event to wait for.
pub struct FdSet {
    bits: [u64; 16],
}
use crate::lang_items::Bytes;
#[allow(unused)]
impl FdSet {
    /// Return an empty bitmap for further manipulation
    pub fn empty() -> Self {
        Self { bits: [0; 16] }
    }
    /// Divide `d` by 64 to decide the `u64` in `bits` to visit.
    fn fd_elt(d: usize) -> usize {
        d >> 6
    }
    /// Mod `d` by 64 for the position of `d` in the `fd_elt()` bitmap.
    fn fd_mask(d: usize) -> u64 {
        1 << (d & 0x3F)
    }
    /// Clear the current struct.
    pub fn clr_all(&mut self) {
        for i in 0..16 {
            self.bits[i] = 0;
        }
    }
    /// Collect all fds with their bits set.
    pub fn get_fd_vec(&self) -> Vec<usize> {
        let mut v = Vec::new();
        for i in 0..1024 {
            if self.is_set(i) {
                v.push(i);
            }
        }
        v
    }
    /// The total number of set bits.
    pub fn set_num(&self) -> u32 {
        let mut sum: u32 = 0;
        for i in self.bits.iter() {
            sum += i.count_ones();
        }
        sum
    }
    pub fn set(&mut self, d: usize) {
        self.bits[Self::fd_elt(d)] |= Self::fd_mask(d);
    }
    /// Clear a certain bit `d` to stop waiting for the event of the correspond fd.
    pub fn clr(&mut self, d: usize) {
        self.bits[Self::fd_elt(d)] &= !Self::fd_mask(d);
    }
    /// Predicate for whether the bit is set for the `d`
    pub fn is_set(&self, d: usize) -> bool {
        (Self::fd_mask(d) & self.bits[Self::fd_elt(d)]) != 0
    }
}
impl Bytes<FdSet> for FdSet {
    fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<FdSet>();
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, size) }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let size = core::mem::size_of::<FdSet>();
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size) }
    }
}
/// Poll each of the file discriptors
/// until certain events.
///
/// # Arguments
///
/// * `nfds`: the highest-numbered file descriptor in any of the three sets
///
/// * `read_fds`: files to be watched to see if characters become available for reading
///
/// * `write_fds`: files to be watched to see if characters become available for writing
///
/// * `except_fds`: exceptional conditions
///
/// (For examples of some exceptional conditions, see the discussion of POLLPRI in [poll(2)].)
/// * `timeout`: argument specifies the interval that pselect() should block waiting for a file descriptor to become ready
///
/// * `sigmask`: the sigmask used by the process during the poll, as in ppoll  
///
/// # Return Value
///
/// * On success, select() and pselect() return the number of file descriptors  contained in the three returned descriptor sets (that is, the total number of bits that are set in  readfds, writefds,  exceptfds)  which  may be zero if the timeout expires before anything interesting happens.  
///
/// * On error, -1  is returned,  the file descriptor sets are unmodified, and  timeout  becomes  undefined.
///  
/// * If both fields of the timeval structure are zero,
///    then select() returns immediately.
///    (This is useful for  polling.)
///    If timeout is NULL (no timeout), select() can block indefinitely.
pub fn pselect(
    nfds: usize,
    read_fds: &mut Option<FdSet>,
    write_fds: &mut Option<FdSet>,
    exception_fds: &mut Option<FdSet>,
    timeout: &Option<TimeSpec>,
    sigmask: *const Signals,
) -> isize {
    let timeout: Option<TimeSpec> = if let Some(ref timeout) = timeout {
        Some(*timeout + crate::timer::TimeSpec::now())
    } else {
        None
    };

    // push to the top of TrapContext page, make use of redundant space
    let oldsig = ((current_task().unwrap().trap_cx_user_va() + crate::config::PAGE_SIZE)
        as *mut Signals)
        .wrapping_sub(1);
    if !sigmask.is_null() {
        sigprocmask(SigMaskHow::SIG_SETMASK.bits(), sigmask, oldsig);
    }

    let mut done = 0;
    loop {
        let task = current_task().unwrap();
        let fd_table = task.files.lock();

        // check read
        if let Some(ref read_fds) = read_fds {
            for i in 0..nfds {
                if !read_fds.is_set(i) {
                    continue;
                }
                if let Ok(fd) = fd_table.get_ref(i) {
                    if fd.r_ready() {
                        done += 1;
                    }
                }
            }
        }
        // check write
        if let Some(ref write_fds) = write_fds {
            for i in 0..nfds {
                if !write_fds.is_set(i) {
                    continue;
                }
                if let Ok(fd) = fd_table.get_ref(i) {
                    if fd.w_ready() {
                        done += 1;
                    }
                }
            }
        }
        // check exception
        // do nothing

        if done != 0 {
            break;
        }
        if let Some(timeout) = timeout {
            if crate::timer::TimeSpec::now() >= timeout {
                break;
            }
        }

        drop(fd_table);
        drop(task);
        suspend_current_and_run_next();
    }
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    // count read
    if let Some(read_fds) = read_fds.as_mut() {
        for i in 0..nfds {
            if !read_fds.is_set(i) {
                continue;
            }
            if let Ok(fd) = fd_table.get_ref(i) {
                if !fd.r_ready() {
                    read_fds.clr(i);
                }
            }
        }
    }
    // count write
    if let Some(write_fds) = write_fds.as_mut() {
        for i in 0..nfds {
            if !write_fds.is_set(i) {
                continue;
            }
            if let Ok(fd) = fd_table.get_ref(i) {
                if !fd.w_ready() {
                    write_fds.clr(i);
                }
            }
        }
    }
    // count exception
    if let Some(exception_fds) = exception_fds {
        *exception_fds = FdSet::empty();
    }
    if !sigmask.is_null() {
        sigprocmask(
            SigMaskHow::SIG_SETMASK.bits(),
            oldsig,
            null_mut::<Signals>(),
        );
    }
    done as isize
}
