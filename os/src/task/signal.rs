use crate::arch::{
    get_bad_instruction, get_exception_cause, MachineContext, TrapContext, UserContext,
};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::{self, Debug, Formatter};
use core::mem::size_of;
use log::{debug, error, trace, warn};

use crate::arch::get_bad_addr;
use crate::mm::{
    copy_from_user, copy_to_user, translated_ref, translated_refmut, try_get_from_user,
};
use crate::syscall::errno::*;
use crate::task::manager::wait_with_timeout;
use crate::task::{block_current_and_run_next, exit_current_and_run_next, exit_group_and_run_next};
use crate::timer::TimeSpec;
use crate::{config::*, signal_type};

use super::current_task;

bitflags! {
    /// Signal
    pub struct Signals: signal_type!(){
        /// Hangup.
        const	SIGHUP		= 1 << ( 0);
        /// Interactive attention signal.
        const	SIGINT		= 1 << ( 1);
        /// Quit.
        const	SIGQUIT		= 1 << ( 2);
        /// Illegal instruction.
        const	SIGILL		= 1 << ( 3);
        /// Trace/breakpoint trap.
        const	SIGTRAP		= 1 << ( 4);
        /// IOT instruction, abort() on a PDP-11.
        const	SIGABRT		= 1 << ( 5);
        /// Bus error.
        const	SIGBUS		= 1 << ( 6);
        /// Erroneous arithmetic operation.
        const	SIGFPE		= 1 << ( 7);
        /// Killed.
        const	SIGKILL		= 1 << ( 8);
        /// User-defined signal 1.
        const	SIGUSR1		= 1 << ( 9);
        /// Invalid access to storage.
        const	SIGSEGV		= 1 << (10);
        /// User-defined signal 2.
        const	SIGUSR2		= 1 << (11);
        /// Broken pipe.
        const	SIGPIPE		= 1 << (12);
        /// Alarm clock.
        const	SIGALRM		= 1 << (13);
        /// Termination request.
        const	SIGTERM		= 1 << (14);
        const	SIGSTKFLT	= 1 << (15);
        /// Child terminated or stopped.
        const	SIGCHLD		= 1 << (16);
        /// Continue.
        const	SIGCONT		= 1 << (17);
        /// Stop, unblockable.
        const	SIGSTOP		= 1 << (18);
        /// Keyboard stop.
        const	SIGTSTP		= 1 << (19);
        /// Background read from control terminal.
        const	SIGTTIN		= 1 << (20);
        /// Background write to control terminal.
        const	SIGTTOU		= 1 << (21);
        /// Urgent data is available at a socket.
        const	SIGURG		= 1 << (22);
        /// CPU time limit exceeded.
        const	SIGXCPU		= 1 << (23);
        /// File size limit exceeded.
        const	SIGXFSZ		= 1 << (24);
        /// Virtual timer expired.
        const	SIGVTALRM	= 1 << (25);
        /// Profiling timer expired.
        const	SIGPROF		= 1 << (26);
        /// Window size change (4.3 BSD, Sun).
        const	SIGWINCH	= 1 << (27);
        /// I/O now possible (4.2 BSD).
        const	SIGIO		= 1 << (28);
        const   SIGPWR      = 1 << (29);
        /// Bad system call.
        const   SIGSYS      = 1 << (30);
        /* --- realtime signals for pthread --- */
        const   SIGTIMER    = 1 << (31);
        const   SIGCANCEL   = 1 << (32);
        const   SIGSYNCCALL = 1 << (33);
        /* --- other realtime signals --- */
        const   SIGRT_3     = 1 << (34);
        const   SIGRT_4     = 1 << (35);
        const   SIGRT_5     = 1 << (36);
        const   SIGRT_6     = 1 << (37);
        const   SIGRT_7     = 1 << (38);
        const   SIGRT_8     = 1 << (39);
        const   SIGRT_9     = 1 << (40);
        const   SIGRT_10    = 1 << (41);
        const   SIGRT_11    = 1 << (42);
        const   SIGRT_12    = 1 << (43);
        const   SIGRT_13    = 1 << (44);
        const   SIGRT_14    = 1 << (45);
        const   SIGRT_15    = 1 << (46);
        const   SIGRT_16    = 1 << (47);
        const   SIGRT_17    = 1 << (48);
        const   SIGRT_18    = 1 << (49);
        const   SIGRT_19    = 1 << (50);
        const   SIGRT_20    = 1 << (51);
        const   SIGRT_21    = 1 << (52);
        const   SIGRT_22    = 1 << (53);
        const   SIGRT_23    = 1 << (54);
        const   SIGRT_24    = 1 << (55);
        const   SIGRT_25    = 1 << (56);
        const   SIGRT_26    = 1 << (57);
        const   SIGRT_27    = 1 << (58);
        const   SIGRT_28    = 1 << (59);
        const   SIGRT_29    = 1 << (60);
        const   SIGRT_30    = 1 << (61);
        const   SIGRT_31    = 1 << (62);
        const   SIGRTMAX    = 1 << (63);
    }
}

impl Signals {
    // SIGILL | SIGKILL | SIGSEGV | SIGSTOP
    const CAN_NOT_BE_MASKED: Signals =
        Signals::from_bits_truncate(1 << 3 | 1 << 8 | 1 << 10 | 1 << 18);
    const EMPTY: Signals = Signals::empty();
    /// if 0 <= signum < 64, return `Ok(Signals)`, else return `Err()` (illeagal)
    pub fn from_signum(signum: usize) -> Result<Signals, ()> {
        match signum {
            0 => Ok(Signals::EMPTY),
            1..=64 => Ok(Signals::from_bits_truncate(1 << (signum - 1))),
            _ => Err(()),
        }
    }
    pub fn to_signum(&self) -> Result<usize, ()> {
        if self.bits().count_ones() == 1 {
            Ok(self.bits().trailing_zeros() as usize + 1)
        } else {
            Err(())
        }
    }
    /// Returns rightmost signal's signum if self is not empty.
    pub fn peek_front(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.bits().trailing_zeros() as usize + 1)
        }
    }
}

bitflags! {
    /// Bits in `sa_flags' used to denote the default signal action.
    pub struct SigActionFlags: usize{
    /// Don't send SIGCHLD when children stop.
        const SA_NOCLDSTOP = 1		   ;
    /// Don't create zombie on child death.
        const SA_NOCLDWAIT = 2		   ;
    /// Invoke signal-catching function with three arguments instead of one.
        const SA_SIGINFO   = 4		   ;
    /// Use signal stack by using `sa_restorer'.
        const SA_ONSTACK   = 0x08000000;
    /// Restart syscall on signal return.
        const SA_RESTART   = 0x10000000;
    /// Don't automatically block the signal when its handler is being executed.
        const SA_NODEFER   = 0x40000000;
    /// Reset to SIG_DFL on entry to handler.
        const SA_RESETHAND = 0x80000000;
    /// Historical no-op.
        const SA_INTERRUPT = 0x20000000;
    /// Use signal trampoline provided by C library's wrapper function.
        const SA_RESTORER  = 0x04000000;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SigHandler(usize);

impl SigHandler {
    /// Default action.
    const SIG_DFL: Self = Self(0);
    /// Ignore signal.
    const SIG_IGN: Self = Self(1);
    fn addr(&self) -> Option<usize> {
        match *self {
            Self::SIG_DFL | Self::SIG_IGN => None,
            sig_handler => Some(sig_handler.0),
        }
    }
}

impl Debug for SigHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            SigHandler::SIG_DFL => f.write_fmt(format_args!("SIG_DFL")),
            SigHandler::SIG_IGN => f.write_fmt(format_args!("SIG_IGN")),
            sig_handler => f.write_fmt(format_args!("0x{:X}", sig_handler.0)),
        }
    }
}

#[cfg(feature = "la64")]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SigAction {
    pub handler: SigHandler,
    pub flags: SigActionFlags,
    pub mask: Signals,
    pub restorer: usize,
}
#[cfg(not(feature = "la64"))]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SigAction {
    pub handler: SigHandler,
    pub flags: SigActionFlags,
    pub restorer: usize,
    pub mask: Signals,
}

impl SigAction {
    pub fn new() -> Self {
        Self {
            handler: SigHandler::SIG_DFL,
            flags: SigActionFlags::empty(),
            restorer: 0,
            mask: Signals::empty(),
        }
    }
}

impl Debug for SigAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "[ sa_handler: {:?}, sa_mask: ({:?}), sa_flags: ({:?}) ]",
            self.handler, self.mask, self.flags
        ))
    }
}

/// Change the action taken by a process on receipt of a specific signal.
/// (See signal(7) for  an  overview of signals.)
/// # Fields in Structure of `act` & `oldact`
///
/// # Arguments
/// * `signum`: specifies the signal and can be any valid signal except `SIGKILL` and `SIGSTOP`.
/// * `act`: new action
/// * `oldact`: old action
pub fn sigaction(signum: usize, act: *const SigAction, oldact: *mut SigAction) -> isize {
    let task = current_task().unwrap();
    match signum {
        0 /* None */ | 9 /* SIGKILL */ | 19 /* SIGSTOP */ | 65.. /* Unsupported */ => {
            warn!("[sigaction] bad signum: {}", signum);
            EINVAL
        }
        signum => {
            trace!("[sigaction] signal: {:?}", Signals::from_signum(signum));
            let token = task.get_user_token();
            if !oldact.is_null() {
                let suc = if let Some(sigact) = &task.sighand.lock()[signum - 1] {
                    trace!("[sigaction] *oldact: {:?}", sigact);
                    copy_to_user(token, sigact.as_ref(), oldact)
                } else {
                    trace!("[sigaction] *oldact: not found");
                    copy_to_user(token, &SigAction::new(), oldact)
                };
                if suc.is_err(){
                    log::error!("[sigaction] Error on copy_to_user(_,{:?},_)",oldact);
                    return EFAULT;
                }
            }
            if !act.is_null() {
                let mut sigact=SigAction::new();
                if copy_from_user(token, act, &mut sigact).is_err(){
                    log::error!("[sigaction] Failed to copy sigact {:?} from user.",act);
                    return EFAULT;
                }
                sigact.mask.remove(Signals::CAN_NOT_BE_MASKED);
                if !(sigact.handler == SigHandler::SIG_DFL || sigact.handler == SigHandler::SIG_IGN)
                {
                    task.sighand.lock()[signum - 1] = Some(Box::new(sigact));
                } else {
                    task.sighand.lock()[signum - 1] = None;
                }
                trace!("[sigaction] *act: {:?}", sigact);
            }
            SUCCESS
        }
    }
}

bitflags! {
    pub struct SignalStackFlags : u32 {
        const ONSTACK = 1;
        const DISABLE = 2;
        const AUTODISARM = 0x80000000;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SignalStack {
    pub sp: usize,
    pub flags: u32,
    pub size: usize,
}

impl SignalStack {
    fn new(sp: usize, size: usize) -> Self {
        SignalStack {
            sp,
            flags: SignalStackFlags::DISABLE.bits,
            size,
        }
    }
}

pub fn do_signal() {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    while let Some(signum) = inner.sigpending.difference(inner.sigmask).peek_front() {
        let signal = Signals::from_bits_truncate(1 << (signum - 1));
        inner.sigpending.remove(signal);
        trace!(
            "[do_signal] signal: {:?}, pending: {:?}, sigmask: {:?}",
            signal,
            inner.sigpending,
            inner.sigmask
        );
        let mut sighand = task.sighand.lock();
        // user-defined handler
        if let Some(act) = &sighand[signum - 1] {
            let trap_cx = inner.get_trap_cx();
            // if this syscall wants to restart
            if get_exception_cause().is_syscall() && trap_cx.gp.a0 == ERESTART as usize {
                // and if `SA_RESTART` is set
                if act.flags.contains(SigActionFlags::SA_RESTART) {
                    debug!("[do_signal] syscall will restart after sigreturn");
                    // back to `ecall`
                    trap_cx.gp.pc -= 4;
                    // restore syscall parameter `a0`
                    trap_cx.gp.a0 = trap_cx.origin_a0;
                } else {
                    debug!("[do_signal] syscall was interrupted");
                    // will return EINTR after sigreturn
                    trap_cx.gp.a0 = EINTR as usize;
                }
            }
            let ucontext_addr = (trap_cx.gp.sp - size_of::<UserContext>()) & !0x7;
            let siginfo_addr = (ucontext_addr - size_of::<SigInfo>()) & !0x7;
            // check if we have enough space on user stack
            let sig_sp = siginfo_addr;
            let sig_size = sig_sp.checked_sub(task.ustack_base - USER_STACK_SIZE);
            if let Some(sig_size) = sig_size {
                let token = task.get_user_token();
                // In this case, signal hander have three parameters
                if act.flags.contains(SigActionFlags::SA_SIGINFO) {
                    copy_to_user(
                        token,
                        &UserContext {
                            flags: 0,
                            link: 0,
                            stack: SignalStack::new(sig_sp, sig_size),
                            sigmask: inner.sigmask,
                            __pad: [0; UserContext::PADDING_SIZE],
                            mcontext: unsafe {
                                *(trap_cx as *const TrapContext).cast::<MachineContext>()
                            },
                        },
                        ucontext_addr as *mut UserContext,
                    ) // push UserContext into user stack
                    .unwrap(); //(This Result was NOT checked and may be usable if left unchecked.)
                    trap_cx.gp.a2 = ucontext_addr; // a2 <- *UserContext
                    copy_to_user(
                        token,
                        &SigInfo::new(signum, 0, 0),
                        siginfo_addr as *mut SigInfo,
                    ) // push SigInfo into user stack
                    .unwrap(); //(This Result was NOT checked and may be usable if left unchecked.)
                    trap_cx.gp.a1 = siginfo_addr; // a1 <- *SigInfo
                                                  // In this case, signal handler only have one parameter (a0 <- signum), so only copy something necessary
                                                  // To simplify the implementation of sigreturn, here we keep the same layout as above...
                } else {
                    *translated_refmut(
                        token,
                        (ucontext_addr + 2 * size_of::<usize>() + size_of::<SignalStack>())
                            as *mut Signals,
                    )
                    .unwrap() = inner.sigmask; // push sigmask into user stack
                    copy_to_user(
                        token,
                        (trap_cx as *const TrapContext).cast::<MachineContext>(),
                        (ucontext_addr
                            + 2 * size_of::<usize>()
                            + size_of::<SignalStack>()
                            + size_of::<Signals>()
                            + UserContext::PADDING_SIZE)
                            as *mut MachineContext,
                    ) // push MachineContext into user stack
                    .unwrap(); //(This Result was NOT checked and may be usable if left unchecked.)
                }
                trap_cx.gp.a0 = signum; // a0 <- signum
                trap_cx.set_sp(sig_sp); // update sp, because we've pushed something into stack
                trap_cx.gp.ra = if act.flags.contains(SigActionFlags::SA_RESTORER) {
                    act.restorer // legacy, signal trampoline provided by C library's wrapper function
                } else {
                    SIGNAL_TRAMPOLINE // ra <- __call_sigreturn, when handler ret, we will go to __call_sigreturn
                };
                trap_cx.gp.pc = act.handler.addr().unwrap(); // restore pc with addr of handler
            } else {
                error!(
                    "[do_signal] User stack will overflow after push trap context! Send SIGSEGV."
                );
                drop(inner);
                drop(sighand);
                drop(task);
                exit_current_and_run_next(Signals::SIGSEGV.to_signum().unwrap() as u32);
            }
            trace!(
                "[do_signal] signal: {:?}, signum: {:?}, handler: {:?} (ra: 0x{:X}, sp: 0x{:X})",
                signal,
                signum,
                act.handler,
                trap_cx.gp.ra,
                trap_cx.gp.sp
            );
            // mask some signals
            inner.sigmask |= if act.flags.contains(SigActionFlags::SA_NODEFER) {
                act.mask - Signals::CAN_NOT_BE_MASKED
            } else {
                (signal | act.mask) - Signals::CAN_NOT_BE_MASKED
            };
            if act.flags.contains(SigActionFlags::SA_RESETHAND) {
                sighand[signum - 1] = None;
            }
            // go back to `trap_return`
            return;
        } else {
            // user program doesn't register a handler for this signal, use our default handler
            match signal {
                // caused by a specific instruction in user program, print log here before exit
                Signals::SIGILL | Signals::SIGSEGV => {
                    let scause = get_exception_cause();
                    if signal == Signals::SIGILL {
                        let stval = get_bad_instruction();
                        warn!("[do_signal] process terminated due to {:?}", signal);
                        println!(
                        "[kernel] {:?} in application, instruction addr = {:#x}, bad instruction = {:#x}, core dumped.",
                        scause,
                        inner.get_trap_cx().gp.pc,
                        stval,
                        );
                    } else {
                        let stval = get_bad_addr();
                        warn!("[do_signal] process terminated due to {:?}", signal);
                        println!(
                        "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                        scause,
                        stval,
                        inner.get_trap_cx().gp.pc,
                        );
                    };
                    drop(inner);
                    drop(sighand);
                    drop(task);
                    exit_group_and_run_next(signal.to_signum().unwrap() as u32);
                }
                // the current process we are handing is sure to be in RUNNING status, so just ignore SIGCONT
                // where we really wake up this process is where we sent SIGCONT, such as `sys_kill()`
                Signals::SIGCHLD | Signals::SIGCONT | Signals::SIGURG | Signals::SIGWINCH => {
                    trace!("[do_signal] Ignore {:?}", signal);
                    continue;
                }
                // stop (or we should say block) current process
                Signals::SIGTSTP | Signals::SIGTTIN | Signals::SIGTTOU => {
                    drop(inner);
                    drop(sighand);
                    drop(task);
                    block_current_and_run_next();
                    // because this loop require `inner`, and we have `drop(inner)` above, so `break` is compulsory
                    // this would cause some signals won't be handled immediately when this process resumes
                    // but it doesn't matter, maybe
                    break;
                }
                // for all other signals, we should terminate current process
                _ => {
                    warn!("[do_signal] process terminated due to {:?}", signal);
                    drop(inner);
                    drop(sighand);
                    drop(task);
                    exit_group_and_run_next(signal.to_signum().unwrap() as u32);
                }
            }
        }
    }
}

bitflags! {
    pub struct SigMaskHow: u32 {
        const SIG_BLOCK     = 0;
        const SIG_UNBLOCK   = 1;
        const SIG_SETMASK   = 2;
    }
}

/// fetch and/or change the signal mask of the calling thread.
/// # Warning
/// In fact, `set` & `oldset` should be 1024 bits `sigset_t`, but we only support 64 signals now.
/// For the sake of performance, we use `Signals` instead.
pub fn sigprocmask(how: u32, set: *const Signals, oldset: *mut Signals) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    let token = task.get_user_token();
    // If oldset is non-NULL, the previous value of the signal mask is stored in oldset
    if oldset as usize != 0 {
        match translated_refmut(token, oldset) {
            Ok(oldset) => *oldset = inner.sigmask,
            Err(errno) => return errno,
        }
        trace!("[sigprocmask] *oldset: ({:?})", inner.sigmask);
    }
    // If set is NULL, then the signal mask is unchanged
    if set as usize != 0 {
        let how = SigMaskHow::from_bits(how);
        let signal_set = match translated_ref(token, set) {
            Ok(set) => *set,
            Err(errno) => return errno,
        };
        trace!("[sigprocmask] how: {:?}, *set: ({:?})", how, signal_set);
        match how {
            // add the signals not yet blocked in the given set to the mask
            Some(SigMaskHow::SIG_BLOCK) => {
                inner.sigmask.insert(signal_set);
            }
            // remove the blocked signals in the set from the sigmask
            // NOTE: unblocking a signal not blocked is allowed
            Some(SigMaskHow::SIG_UNBLOCK) => {
                inner.sigmask.remove(signal_set);
            }
            // set the signal mask to what we see
            Some(SigMaskHow::SIG_SETMASK) => {
                inner.sigmask = signal_set;
            }
            // `how` was invalid
            _ => return EINVAL,
        };
        // unblock SIGILL & SIGSEGV, otherwise infinite loop may occurred
        // unblock SIGKILL & SIGSTOP, they can't be masked according to standard
        inner.sigmask.remove(Signals::CAN_NOT_BE_MASKED);
    }
    SUCCESS
}

#[allow(unused)]
#[derive(Clone, Copy)]
#[repr(C)] //UNSAFE! IS THIS CORRECT?
pub struct SigInfo {
    si_signo: u32,
    si_errno: u32,
    si_code: u32,
    // unsupported fields
    __pad: [u8; 128 - 3 * core::mem::size_of::<u32>()],
}

impl SigInfo {
    pub fn new(si_signo: usize, si_errno: usize, si_code: usize) -> Self {
        Self {
            si_signo: si_signo as u32,
            si_errno: si_errno as u32,
            si_code: si_code as u32,
            __pad: [0; 128 - 3 * core::mem::size_of::<u32>()],
        }
    }
}

#[allow(unused)]
impl SigInfo {
    const SI_ASYNCNL: u32 = 60u32.wrapping_neg();
    const SI_TKILL: u32 = 6u32.wrapping_neg();
    const SI_SIGIO: u32 = 5u32.wrapping_neg();
    const SI_ASYNCIO: u32 = 4u32.wrapping_neg();
    const SI_MESGQ: u32 = 3u32.wrapping_neg();
    const SI_TIMER: u32 = 2u32.wrapping_neg();
    const SI_QUEUE: u32 = 1u32.wrapping_neg();
    const SI_USER: u32 = 0;
    const SI_KERNEL: u32 = 128;
    const FPE_INTDIV: u32 = 1;
    const FPE_INTOVF: u32 = 2;
    const FPE_FLTDIV: u32 = 3;
    const FPE_FLTOVF: u32 = 4;
    const FPE_FLTUND: u32 = 5;
    const FPE_FLTRES: u32 = 6;
    const FPE_FLTINV: u32 = 7;
    const FPE_FLTSUB: u32 = 8;
    const ILL_ILLOPC: u32 = 1;
    const ILL_ILLOPN: u32 = 2;
    const ILL_ILLADR: u32 = 3;
    const ILL_ILLTRP: u32 = 4;
    const ILL_PRVOPC: u32 = 5;
    const ILL_PRVREG: u32 = 6;
    const ILL_COPROC: u32 = 7;
    const ILL_BADSTK: u32 = 8;
    const SEGV_MAPERR: u32 = 1;
    const SEGV_ACCERR: u32 = 2;
    const SEGV_BNDERR: u32 = 3;
    const SEGV_PKUERR: u32 = 4;
    const BUS_ADRALN: u32 = 1;
    const BUS_ADRERR: u32 = 2;
    const BUS_OBJERR: u32 = 3;
    const BUS_MCEERR_AR: u32 = 4;
    const BUS_MCEERR_AO: u32 = 5;
    const CLD_EXITED: u32 = 1;
    const CLD_KILLED: u32 = 2;
    const CLD_DUMPED: u32 = 3;
    const CLD_TRAPPED: u32 = 4;
    const CLD_STOPPED: u32 = 5;
    const CLD_CONTINUED: u32 = 6;
}

pub fn sigtimedwait(set: *const Signals, info: *mut SigInfo, timeout: *const TimeSpec) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let set = match translated_ref(token, set) {
        Ok(set) => *set,
        Err(errno) => return errno,
    };
    let timeout = match try_get_from_user(token, timeout) {
        Ok(timeout) => match timeout {
            Some(timeout) => timeout,
            None => return EINVAL,
        },
        Err(errno) => return errno,
    };
    debug!("[sigtimedwait] set: {:?}, timeout: {:?}", set, timeout);

    let start = TimeSpec::now();
    wait_with_timeout(Arc::downgrade(&task), start + timeout);
    drop(task);

    block_current_and_run_next();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    // interrupted by signal(s)
    if !inner.sigpending.is_empty() {
        match (inner.sigpending & set).peek_front() {
            Some(signum) => {
                if !info.is_null() {
                    if copy_to_user(token, &SigInfo::new(signum, 0, 0), info).is_err() {
                        log::error!("[sys_sigtimedwait] Error copying to info {:?} ", info);
                        return EFAULT;
                    };
                }
                signum as isize
            }
            // Interrupted by signal(s) that not present in `set`
            // This syscall is never restarted after being interrupted by a signal handler
            None => EINTR,
        }
    // reach timeout
    } else {
        assert!(start + timeout <= TimeSpec::now());
        EAGAIN
    }
}
