pub mod errno;
pub mod fs;
mod process;
// mod socket;

use crate::arch::syscall_id::*;
use core::convert::TryFrom;
use fs::*;
use log::{error, info};
pub use process::CloneFlags;
use process::*;
// pub use process::{CloneFlags, FutexOption};
// use socket::*;

pub fn syscall_name(id: usize) -> &'static str {
    match id {
        SYSCALL_DUP => "dup",
        SYSCALL_DUP3 => "dup3",
        SYSCALL_OPEN => "open",
        SYSCALL_GET_TIME => "get_time",
        SYSCALL_GETCWD => "getcwd",
        SYSCALL_FCNTL => "fcntl",
        SYSCALL_IOCTL => "ioctl",
        SYSCALL_MKDIRAT => "mkdirat",
        SYSCALL_UNLINKAT => "unlinkat",
        SYSCALL_LINKAT => "linkat",
        SYSCALL_UMOUNT2 => "umount2",
        SYSCALL_MOUNT => "mount",
        SYSCALL_FACCESSAT => "faccessat",
        SYSCALL_CHDIR => "chdir",
        SYSCALL_OPENAT => "openat",
        SYSCALL_CLOSE => "close",
        SYSCALL_PIPE2 => "pipe2",
        SYSCALL_GETDENTS64 => "getdents64",
        SYSCALL_LSEEK => "lseek",
        SYSCALL_READ => "read",
        SYSCALL_WRITE => "write",
        SYSCALL_READV => "readv",
        SYSCALL_WRITEV => "writev",
        SYSCALL_PREAD => "pread",
        SYSCALL_PWRITE => "pwrite",
        SYSCALL_SENDFILE => "sendfile",
        SYSCALL_PSELECT6 => "pselect6",
        SYSCALL_PPOLL => "ppoll",
        SYSCALL_READLINKAT => "readlinkat",
        SYSCALL_FSTATAT => "fstatat",
        SYSCALL_FSTAT => "fstat",
        SYSCALL_STATX => "statx",
        SYSCALL_STATFS => "statfs",
        SYSCALL_FTRUNCATE => "ftruncate",
        SYSCALL_FSYNC => "fsync",
        SYSCALL_UTIMENSAT => "utimensat",
        SYSCALL_EXIT => "exit",
        SYSCALL_EXIT_GROUP => "exit_GROUP",
        SYSCALL_SET_TID_ADDRESS => "set_tid_address",
        SYSCALL_FUTEX => "futex",
        SYSCALL_SET_ROBUST_LIST => "set_robust_list",
        SYSCALL_GET_ROBUST_LIST => "get_robust_list",
        SYSCALL_NANOSLEEP => "nanosleep",
        SYSCALL_GETITIMER => "getitimer",
        SYSCALL_SETITIMER => "setitimer",
        SYSCALL_CLOCK_GETTIME => "clock_gettime",
        SYSCALL_SYSLOG => "syslog",
        SYSCALL_YIELD => "yield",
        SYSCALL_KILL => "kill",
        SYSCALL_TKILL => "tkill",
        SYSCALL_SIGACTION => "sigaction",
        SYSCALL_SIGPROCMASK => "sigprocmask",
        SYSCALL_SIGTIMEDWAIT => "sigtimedwait",
        SYSCALL_SIGRETURN => "sigreturn",
        SYSCALL_TIMES => "times",
        SYSCALL_SETPGID => "setpgid",
        SYSCALL_GETPGID => "getpgid",
        SYSCALL_UNAME => "uname",
        SYSCALL_GETRUSAGE => "getrusage",
        SYSCALL_UMASK => "umask",
        SYSCALL_GET_TIME_OF_DAY => "get_time_of_day",
        SYSCALL_GETPID => "getpid",
        SYSCALL_GETPPID => "getppid",
        SYSCALL_GETUID => "getuid",
        SYSCALL_GETEUID => "geteuid",
        SYSCALL_GETGID => "getgid",
        SYSCALL_GETEGID => "getegid",
        SYSCALL_GETTID => "gettid",
        SYSCALL_SYSINFO => "sysinfo",
        SYSCALL_SOCKET => "socket",
        SYSCALL_BIND => "bind",
        SYSCALL_LISTEN => "listen",
        SYSCALL_ACCEPT => "accept",
        SYSCALL_CONNECT => "connect",
        SYSCALL_GETSOCKNAME => "getsockname",
        SYSCALL_GETPEERNAME => "getpeername",
        SYSCALL_SENDTO => "sendto",
        SYSCALL_RECVFROM => "recvfrom",
        SYSCALL_SETSOCKOPT => "setsockopt",
        SYSCALL_SBRK => "sbrk",
        SYSCALL_BRK => "brk",
        SYSCALL_MUNMAP => "munmap",
        SYSCALL_CLONE => "clone",
        SYSCALL_EXECVE => "execve",
        SYSCALL_MMAP => "mmap",
        SYSCALL_MPROTECT => "mprotect",
        SYSCALL_MSYNC => "msync",
        SYSCALL_WAIT4 => "wait4",
        SYSCALL_PRLIMIT => "prlimit",
        SYSCALL_RENAMEAT2 => "renameat2",
        SYSCALL_FACCESSAT2 => "faccessat2",
        SYSCALL_MEMBARRIER => "membarrier",
        // non-standard
        SYSCALL_LS => "ls",
        SYSCALL_SHUTDOWN => "shutdown",
        SYSCALL_CLEAR => "clear",
        _ => "unknown",
    }
}
use crate::{
    fs::poll::FdSet,
    syscall::errno::Errno,
    task::Rusage,
    timer::{ITimerVal, TimeSpec, Times},
};

pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    let mut show_info = false;
    if option_env!("LOG").is_some()
        && ![
            //black list
            SYSCALL_YIELD,
            SYSCALL_READ,
            SYSCALL_WRITE,
            SYSCALL_GETDENTS64,
            SYSCALL_READV,
            SYSCALL_WRITEV,
            SYSCALL_PSELECT6,
            SYSCALL_SIGACTION,
            SYSCALL_SIGPROCMASK,
            // SYSCALL_WAIT4,
            // SYSCALL_GETPPID,
            SYSCALL_CLOCK_GETTIME,
        ]
        .contains(&syscall_id)
    {
        show_info = true;
        info!(
            "[syscall] {}({}) args: [{:X}, {:X}, {:X}, {:X}, {:X}, {:X}]",
            syscall_name(syscall_id),
            syscall_id,
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
            args[5],
        );
    }
    let ret = match syscall_id {
        SYSCALL_GETCWD => sys_getcwd(args[0], args[1]),
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_DUP3 => sys_dup3(args[0], args[1], args[2] as u32),
        SYSCALL_FCNTL => sys_fcntl(args[0], args[1] as u32, args[2]),
        SYSCALL_IOCTL => sys_ioctl(args[0], args[1] as u32, args[2]),
        SYSCALL_MKDIRAT => sys_mkdirat(args[0], args[1] as *const u8, args[2] as u32),
        SYSCALL_UNLINKAT => sys_unlinkat(args[0], args[1] as *const u8, args[2] as u32),
        SYSCALL_UMOUNT2 => sys_umount2(args[0] as *const u8, args[1] as u32),
        SYSCALL_MOUNT => sys_mount(
            args[0] as *const u8,
            args[1] as *const u8,
            args[2] as *const u8,
            args[3],
            args[4] as *const u8,
        ),
        SYSCALL_FACCESSAT => sys_faccessat2(args[0], args[1] as *const u8, args[2] as u32, 0u32),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_OPEN => sys_openat(AT_FDCWD, args[0] as *const u8, args[1] as u32, 0o777u32),
        SYSCALL_OPENAT => sys_openat(
            args[0],
            args[1] as *const u8,
            args[2] as u32,
            args[3] as u32,
        ),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE2 => sys_pipe2(args[0], args[1] as u32),
        SYSCALL_GETDENTS64 => sys_getdents64(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1], args[2]),
        SYSCALL_READV => sys_readv(args[0], args[1], args[2]),
        SYSCALL_PREAD => sys_pread(args[0], args[1], args[2], args[3]),
        SYSCALL_WRITE => sys_write(args[0], args[1], args[2]),
        SYSCALL_WRITEV => sys_writev(args[0], args[1], args[2]),
        SYSCALL_PWRITE => sys_pwrite(args[0], args[1], args[2], args[3]),
        SYSCALL_LSEEK => sys_lseek(args[0], args[1] as isize, args[2] as u32),
        SYSCALL_SENDFILE => sys_sendfile(args[0], args[1], args[2] as *mut usize, args[3]),
        SYSCALL_READLINKAT => {
            sys_readlinkat(args[0], args[1] as *const u8, args[2] as *mut u8, args[3])
        }
        SYSCALL_FSTATAT => sys_fstatat(
            args[0],
            args[1] as *const u8,
            args[2] as *mut u8,
            args[3] as u32,
        ),
        SYSCALL_FSTAT => sys_fstat(args[0], args[1] as *mut u8),
        SYSCALL_STATX => sys_statx(
            args[0],
            args[1] as *const u8,
            args[2] as u32,
            args[3] as u32,
            args[4] as *mut u8,
        ),
        SYSCALL_FTRUNCATE => sys_ftruncate(args[0], args[1] as isize),
        SYSCALL_FSYNC => sys_fsync(args[0]),
        SYSCALL_UTIMENSAT => sys_utimensat(
            args[0],
            args[1] as *const u8,
            args[2] as *const [TimeSpec; 2],
            args[3] as u32,
        ),
        SYSCALL_EXIT => sys_exit(args[0] as u32),
        SYSCALL_EXIT_GROUP => sys_exit_group(args[0] as u32),
        SYSCALL_CLOCK_GETTIME => sys_clock_gettime(args[0], args[1] as *mut TimeSpec),
        SYSCALL_KILL => sys_kill(args[0], args[1]),
        SYSCALL_TKILL => sys_tkill(args[0], args[1]),
        SYSCALL_SYSLOG => sys_syslog(args[0] as u32, args[1] as *mut u8, args[2] as u32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_SIGACTION => sys_sigaction(args[0], args[1], args[2]),
        SYSCALL_SIGPROCMASK => sys_sigprocmask(args[0] as u32, args[1], args[2]),
        SYSCALL_SIGTIMEDWAIT => sys_sigtimedwait(args[0], args[1], args[2]),
        SYSCALL_SIGRETURN => sys_sigreturn(),
        SYSCALL_TIMES => sys_times(args[0] as *mut Times),
        SYSCALL_NANOSLEEP => sys_nanosleep(
            args[0] as *const crate::timer::TimeSpec,
            args[1] as *mut crate::timer::TimeSpec,
        ),
        SYSCALL_SETITIMER => sys_setitimer(
            args[0],
            args[1] as *const ITimerVal,
            args[2] as *mut ITimerVal,
        ),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETRUSAGE => sys_getrusage(args[0] as isize, args[1] as *mut Rusage),
        SYSCALL_UMASK => sys_umask(args[0] as u32),
        SYSCALL_GET_TIME_OF_DAY => sys_gettimeofday(
            args[0] as *mut crate::timer::TimeVal,
            args[1] as *mut crate::timer::TimeZone,
        ),
        SYSCALL_SETPGID => sys_setpgid(args[0], args[1]),
        SYSCALL_GETPGID => sys_getpgid(args[0]),
        SYSCALL_UNAME => sys_uname(args[0] as *mut u8),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_GETPPID => sys_getppid(),
        SYSCALL_CLONE => sys_clone(
            args[0] as u32,
            args[1] as *const u8,
            args[2] as *mut u32,
            args[3],
            args[4] as *mut u32,
        ),
        SYSCALL_EXECVE => sys_execve(
            args[0] as *const u8,
            args[1] as *const *const u8,
            args[2] as *const *const u8,
        ),
        SYSCALL_WAIT4 => sys_wait4(
            args[0] as isize,
            args[1] as *mut u32,
            args[2] as u32,
            args[3] as *mut Rusage,
        ),
        SYSCALL_PRLIMIT => sys_prlimit(
            args[0],
            args[1] as u32,
            args[2] as *const RLimit,
            args[3] as *mut RLimit,
        ),
        SYSCALL_SET_TID_ADDRESS => sys_set_tid_address(args[0]),
        SYSCALL_FUTEX => sys_futex(
            args[0] as *mut u32,
            args[1] as u32,
            args[2] as u32,
            args[3] as *const TimeSpec,
            args[4] as *mut u32,
            args[5] as u32,
        ),
        SYSCALL_SET_ROBUST_LIST => sys_set_robust_list(args[0], args[1]),
        SYSCALL_GET_ROBUST_LIST => {
            sys_get_robust_list(args[0] as u32, args[1] as *mut usize, args[2] as *mut usize)
        }
        SYSCALL_GETUID => sys_getuid(),
        SYSCALL_GETEUID => sys_geteuid(),
        SYSCALL_GETGID => sys_getgid(),
        SYSCALL_GETEGID => sys_getegid(),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_SYSINFO => sys_sysinfo(args[0] as *mut Sysinfo),
        SYSCALL_SBRK => sys_sbrk(args[0] as isize),
        SYSCALL_BRK => sys_brk(args[0]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2], args[3], args[4], args[5]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_MPROTECT => sys_mprotect(args[0], args[1], args[2]),
        SYSCALL_PSELECT6 => sys_pselect(
            args[0],
            args[1] as *mut FdSet,
            args[2] as *mut FdSet,
            args[3] as *mut FdSet,
            args[4] as *mut TimeSpec,
            args[5] as *const crate::task::Signals,
        ),
        SYSCALL_PPOLL => sys_ppoll(args[0], args[1], args[2], args[3]),
        SYSCALL_FACCESSAT2 => sys_faccessat2(
            args[0],
            args[1] as *const u8,
            args[2] as u32,
            args[3] as u32,
        ),
        SYSCALL_MEMBARRIER => sys_memorybarrier(args[0], args[1], args[2]),
        SYSCALL_RENAMEAT2 => sys_renameat2(
            args[0],
            args[1] as *const u8,
            args[2],
            args[3] as *const u8,
            args[4] as u32,
        ),
        SYSCALL_MSYNC => sys_msync(args[0], args[1], args[2] as u32),
        SYSCALL_STATFS => sys_statfs(args[0] as *const u8, args[1] as *mut Statfs),
        // SYSCALL_SOCKET => sys_socket(args[0] as u32, args[1] as u32, args[2] as u32),
        // SYSCALL_BIND => sys_bind(args[0], args[1] as *const u8, args[2] as u32),
        // SYSCALL_LISTEN => sys_listen(args[0], args[1] as u32),
        // SYSCALL_ACCEPT => sys_accept(args[0], args[1] as *const u8, args[2] as u32),
        // SYSCALL_CONNECT => sys_connect(args[0], args[1] as *const u8, args[2] as u32),
        // SYSCALL_GETSOCKNAME => sys_getsockname(args[0], args[1] as *const u8, args[2] as u32),
        // SYSCALL_GETPEERNAME => sys_getpeername(args[0], args[1] as *const u8, args[2] as u32),
        // SYSCALL_SENDTO => sys_sendto(
        //     args[0],
        //     args[1] as *const u8,
        //     args[2],
        //     args[3] as u32,
        //     args[4] as *const u8,
        //     args[5] as u32,
        // ),
        // SYSCALL_RECVFROM => sys_recvfrom(
        //     args[0],
        //     args[1] as *mut u8,
        //     args[2],
        //     args[3] as u32,
        //     args[4] as *const u8,
        //     args[5] as u32,
        // ),
        // SYSCALL_SETSOCKOPT => sys_setsockopt(
        //     args[0],
        //     args[1] as u32,
        //     args[2] as u32,
        //     args[3] as *const u8,
        //     args[4] as u32,
        // ),
        SYSCALL_SHUTDOWN => sys_shutdown(),
        _ => {
            error!(
                "Unsupported syscall:{} ({}), calling over arguments:",
                syscall_name(syscall_id),
                syscall_id
            );
            for i in 0..args.len() {
                error!("args[{}]: {:X}", i, args[i]);
            }
            crate::task::current_task()
                .unwrap()
                .acquire_inner_lock()
                .add_signal(crate::task::Signals::SIGSYS);
            errno::ENOSYS
        }
    };

    if option_env!("LOG").is_some() && show_info {
        match Errno::try_from(ret) {
            Ok(errno) => info!(
                "[syscall] {}({}) -> {:?}",
                syscall_name(syscall_id),
                syscall_id,
                errno
            ),
            Err(val) => info!(
                "[syscall] {}({}) -> {:X}",
                syscall_name(syscall_id),
                syscall_id,
                val.number
            ),
        }
    }
    ret
}
