use super::manager::TASK_MANAGER;
use super::pid::{kstack_alloc, RecycleAllocator};
use super::signal::*;
use super::threads::Futex;
use super::trap_cx_bottom_from_tid;
use super::ustack_bottom_from_tid;
use super::TaskContext;
use super::{pid_alloc, KernelStackImpl, PidHandle};
use crate::arch::TrapImpl;
use crate::arch::{trap_handler, TrapContext};
use crate::config::MMAP_BASE;
use crate::fs::{FdTable, FileDescriptor, OpenFlags, ROOT_FD};
use crate::mm::{MemorySet, PageTableImpl, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::syscall::CloneFlags;
use crate::timer::{ITimerVal, TimeVal};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use log::trace;
use spin::{Mutex, MutexGuard};

#[derive(Clone)]
pub struct FsStatus {
    pub working_inode: Arc<FileDescriptor>,
}

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub tid: usize,
    pub tgid: usize,
    pub kstack: KernelStackImpl,
    pub ustack_base: usize,
    pub exit_signal: Signals,
    // mutable
    inner: Mutex<TaskControlBlockInner>,
    // shareable and mutable
    pub exe: Arc<Mutex<FileDescriptor>>,
    pub tid_allocator: Arc<Mutex<RecycleAllocator>>,
    pub files: Arc<Mutex<FdTable>>,
    pub fs: Arc<Mutex<FsStatus>>,
    pub vm: Arc<Mutex<MemorySet<PageTableImpl>>>,
    pub sighand: Arc<Mutex<Vec<Option<Box<SigAction>>>>>,
    pub futex: Arc<Mutex<Futex>>,
}

pub struct TaskControlBlockInner {
    pub sigmask: Signals,
    pub sigpending: Signals,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: u32,
    pub clear_child_tid: usize,
    pub robust_list: RobustList,
    pub heap_bottom: usize,
    pub heap_pt: usize,
    pub pgid: usize,
    pub rusage: Rusage,
    pub clock: ProcClock,
    pub timer: [ITimerVal; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct RobustList {
    pub head: usize,
    pub len: usize,
}

impl RobustList {
    // from strace
    pub const HEAD_SIZE: usize = 24;
}

impl Default for RobustList {
    fn default() -> Self {
        Self {
            head: 0,
            len: Self::HEAD_SIZE,
        }
    }
}

#[repr(C)]
pub struct ProcClock {
    last_enter_u_mode: TimeVal,
    last_enter_s_mode: TimeVal,
}

impl ProcClock {
    pub fn new() -> Self {
        let now = TimeVal::now();
        Self {
            last_enter_u_mode: now,
            last_enter_s_mode: now,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Rusage {
    pub ru_utime: TimeVal, /* user CPU time used */
    pub ru_stime: TimeVal, /* system CPU time used */
    ru_maxrss: isize,      // NOT IMPLEMENTED /* maximum resident set size */
    ru_ixrss: isize,       // NOT IMPLEMENTED /* integral shared memory size */
    ru_idrss: isize,       // NOT IMPLEMENTED /* integral unshared data size */
    ru_isrss: isize,       // NOT IMPLEMENTED /* integral unshared stack size */
    ru_minflt: isize,      // NOT IMPLEMENTED /* page reclaims (soft page faults) */
    ru_majflt: isize,      // NOT IMPLEMENTED /* page faults (hard page faults) */
    ru_nswap: isize,       // NOT IMPLEMENTED /* swaps */
    ru_inblock: isize,     // NOT IMPLEMENTED /* block input operations */
    ru_oublock: isize,     // NOT IMPLEMENTED /* block output operations */
    ru_msgsnd: isize,      // NOT IMPLEMENTED /* IPC messages sent */
    ru_msgrcv: isize,      // NOT IMPLEMENTED /* IPC messages received */
    ru_nsignals: isize,    // NOT IMPLEMENTED /* signals received */
    ru_nvcsw: isize,       // NOT IMPLEMENTED /* voluntary context switches */
    ru_nivcsw: isize,      // NOT IMPLEMENTED /* involuntary context switches */
}

impl Rusage {
    pub fn new() -> Self {
        Self {
            ru_utime: TimeVal::new(),
            ru_stime: TimeVal::new(),
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        }
    }
}

impl Debug for Rusage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "(ru_utime:{:?}, ru_stime:{:?})",
            self.ru_utime, self.ru_stime
        ))
    }
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn add_signal(&mut self, signal: Signals) {
        self.sigpending.insert(signal);
    }
    pub fn update_process_times_enter_trap(&mut self) {
        let now = TimeVal::now();
        self.clock.last_enter_s_mode = now;
        let diff = now - self.clock.last_enter_u_mode;
        self.rusage.ru_utime = self.rusage.ru_utime + diff;
        self.update_itimer_virtual_if_exists(diff);
        self.update_itimer_prof_if_exists(diff);
    }
    pub fn update_process_times_leave_trap(&mut self, trap_cause: TrapImpl) {
        let now = TimeVal::now();
        self.update_itimer_real_if_exists(now - self.clock.last_enter_u_mode);
        if trap_cause.is_timer() {
            let diff = now - self.clock.last_enter_s_mode;
            self.rusage.ru_stime = self.rusage.ru_stime + diff;
            self.update_itimer_prof_if_exists(diff);
        }
        self.clock.last_enter_u_mode = now;
    }
    pub fn update_itimer_real_if_exists(&mut self, diff: TimeVal) {
        if !self.timer[0].it_value.is_zero() {
            self.timer[0].it_value = self.timer[0].it_value - diff;
            if self.timer[0].it_value.is_zero() {
                self.add_signal(Signals::SIGALRM);
                self.timer[0].it_value = self.timer[0].it_interval;
            }
        }
    }
    pub fn update_itimer_virtual_if_exists(&mut self, diff: TimeVal) {
        if !self.timer[1].it_value.is_zero() {
            self.timer[1].it_value = self.timer[1].it_value - diff;
            if self.timer[1].it_value.is_zero() {
                self.add_signal(Signals::SIGVTALRM);
                self.timer[1].it_value = self.timer[1].it_interval;
            }
        }
    }
    pub fn update_itimer_prof_if_exists(&mut self, diff: TimeVal) {
        if !self.timer[2].it_value.is_zero() {
            self.timer[2].it_value = self.timer[2].it_value - diff;
            if self.timer[2].it_value.is_zero() {
                self.add_signal(Signals::SIGPROF);
                self.timer[2].it_value = self.timer[2].it_interval;
            }
        }
    }
}

impl TaskControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }
    pub fn trap_cx_user_va(&self) -> usize {
        trap_cx_bottom_from_tid(self.tid)
    }
    pub fn ustack_bottom_va(&self) -> usize {
        ustack_bottom_from_tid(self.tid)
    }
    /// !!!!!!!!!!!!!!!!WARNING!!!!!!!!!!!!!!!!!!!!!
    /// Currently used for initproc loading only. bin_path must be used changed if used elsewhere.
    pub fn new(elf: FileDescriptor) -> Self {
        let elf_data = elf.map_to_kernel_space(MMAP_BASE);
        // memory_set with elf program headers/trampoline
        let (mut memory_set, user_heap, elf_info) = MemorySet::from_elf(elf_data).unwrap();
        crate::mm::KERNEL_SPACE
            .lock()
            .remove_area_with_start_vpn(VirtAddr::from(MMAP_BASE).floor())
            .unwrap();

        let tid_allocator = Arc::new(Mutex::new(RecycleAllocator::new()));
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let tid = tid_allocator.lock().alloc();
        let tgid = pid_handle.0;
        let pgid = pid_handle.0;
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();

        memory_set.alloc_user_res(tid, true);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(trap_cx_bottom_from_tid(tid)).into())
            .unwrap();
        log::trace!("[TCB::new]trap_cx_ppn{:?}", trap_cx_ppn);
        let task_control_block = Self {
            pid: pid_handle,
            tid,
            tgid,
            kstack,
            ustack_base: ustack_bottom_from_tid(tid),
            exit_signal: Signals::empty(),
            exe: Arc::new(Mutex::new(elf)),
            tid_allocator,
            files: Arc::new(Mutex::new(FdTable::new({
                let mut vec = Vec::with_capacity(144);
                let tty = Some(ROOT_FD.open("/dev/tty", OpenFlags::O_RDWR, false).unwrap());
                vec.resize(3, tty);
                vec
            }))),
            fs: Arc::new(Mutex::new(FsStatus {
                working_inode: Arc::new(
                    ROOT_FD
                        .open(".", OpenFlags::O_RDONLY | OpenFlags::O_DIRECTORY, true)
                        .unwrap(),
                ),
            })),
            vm: Arc::new(Mutex::new(memory_set)),
            sighand: Arc::new(Mutex::new({
                let mut vec = Vec::with_capacity(64);
                vec.resize(64, None);
                vec
            })),
            futex: Arc::new(Mutex::new(Futex::new())),
            inner: Mutex::new(TaskControlBlockInner {
                sigmask: Signals::empty(),
                sigpending: Signals::empty(),
                trap_cx_ppn,
                task_cx: TaskContext::goto_trap_return(kstack_top),
                task_status: TaskStatus::Ready,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
                clear_child_tid: 0,
                robust_list: RobustList::default(),
                heap_bottom: user_heap,
                heap_pt: user_heap,
                pgid,
                rusage: Rusage::new(),
                clock: ProcClock::new(),
                timer: [ITimerVal::new(); 3],
            }),
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            elf_info.entry,
            ustack_bottom_from_tid(tid),
            KERNEL_SPACE.lock().token(),
            kstack_top,
            trap_handler as usize,
        );
        trace!("[new] trap_cx:{:?}", *trap_cx);
        task_control_block
    }

    pub fn load_elf(
        &self,
        elf: FileDescriptor,
        argv_vec: &Vec<String>,
        envp_vec: &Vec<String>,
    ) -> Result<(), isize> {
        let elf_data = elf.map_to_kernel_space(MMAP_BASE);
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut memory_set, program_break, elf_info) = MemorySet::from_elf(elf_data)?;
        log::trace!("[load_elf] ELF file mapped");
        // remove elf area
        crate::mm::KERNEL_SPACE
            .lock()
            .remove_area_with_start_vpn(VirtAddr::from(MMAP_BASE).floor())
            .unwrap();
        memory_set.alloc_user_res(self.tid, true);
        let user_sp =
            memory_set.create_elf_tables(self.ustack_bottom_va(), argv_vec, envp_vec, &elf_info);
        log::trace!("[load_elf] user sp after pushing parameters: {:X}", user_sp);
        // initialize trap_cx
        let trap_cx = TrapContext::app_init_context(
            if let Some(interp_entry) = elf_info.interp_entry {
                interp_entry
            } else {
                elf_info.entry
            },
            user_sp,
            KERNEL_SPACE.lock().token(),
            self.kstack.get_top(),
            trap_handler as usize,
        );
        // **** hold current PCB lock
        let mut inner = self.acquire_inner_lock();
        // update trap_cx ppn
        inner.trap_cx_ppn = (&memory_set)
            .translate(VirtAddr::from(self.trap_cx_user_va()).into())
            .unwrap();
        *inner.get_trap_cx() = trap_cx;
        // clear clear_child_tid
        inner.clear_child_tid = 0;
        // clear robust_list
        inner.robust_list = RobustList::default();
        // update heap pointers
        inner.heap_bottom = program_break;
        inner.heap_pt = program_break;
        // track the change of ELF file
        *self.exe.lock() = elf;
        // flush cloexec fd
        self.files.lock().iter_mut().for_each(|fd| match fd {
            Some(file) => {
                if file.get_cloexec() {
                    *fd = None;
                }
            }
            None => (),
        });
        // substitute memory_set
        *self.vm.lock() = memory_set;
        // flush signal handler
        for sigact in self.sighand.lock().iter_mut() {
            *sigact = None;
        }
        // flush futex
        self.futex.lock().clear();
        if self.tid_allocator.lock().get_allocated() > 1 {
            let mut manager = TASK_MANAGER.lock();
            // destory all other threads
            manager
                .ready_queue
                .retain(|task| (*task).tgid != (*self).tgid);
            manager
                .interruptible_queue
                .retain(|task| (*task).tgid != (*self).tgid);
        };
        Ok(())
        // **** release current PCB lock
    }
    pub fn sys_clone(
        self: &Arc<TaskControlBlock>,
        flags: CloneFlags,
        stack: *const u8,
        tls: usize,
        exit_signal: Signals,
    ) -> Arc<TaskControlBlock> {
        // ---- hold parent PCB lock
        let mut parent_inner = self.acquire_inner_lock();
        // copy user space(include trap context)
        let memory_set = if flags.contains(CloneFlags::CLONE_VM) {
            self.vm.clone()
        } else {
            crate::mm::frame_reserve(16);
            Arc::new(Mutex::new(MemorySet::from_existing_user(
                &mut self.vm.lock(),
            )))
        };

        let tid_allocator = if flags.contains(CloneFlags::CLONE_THREAD) {
            self.tid_allocator.clone()
        } else {
            Arc::new(Mutex::new(RecycleAllocator::new()))
        };
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let tid = tid_allocator.lock().alloc();
        let tgid = if flags.contains(CloneFlags::CLONE_THREAD) {
            self.tgid
        } else {
            pid_handle.0
        };
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();

        if flags.contains(CloneFlags::CLONE_THREAD) {
            memory_set.lock().alloc_user_res(tid, stack.is_null());
        }
        let trap_cx_ppn = memory_set
            .lock()
            .translate(VirtAddr::from(trap_cx_bottom_from_tid(tid)).into())
            .unwrap();

        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            tid,
            tgid,
            kstack,
            ustack_base: if !stack.is_null() {
                stack as usize
            } else {
                ustack_bottom_from_tid(tid)
            },
            exit_signal,
            exe: self.exe.clone(),
            tid_allocator,
            files: if flags.contains(CloneFlags::CLONE_FILES) {
                self.files.clone()
            } else {
                Arc::new(Mutex::new(self.files.lock().clone()))
            },
            fs: if flags.contains(CloneFlags::CLONE_FS) {
                self.fs.clone()
            } else {
                Arc::new(Mutex::new(self.fs.lock().clone()))
            },
            vm: memory_set,
            sighand: if flags.contains(CloneFlags::CLONE_SIGHAND) {
                self.sighand.clone()
            } else {
                Arc::new(Mutex::new(self.sighand.lock().clone()))
            },
            futex: if flags.contains(CloneFlags::CLONE_SYSVSEM) {
                self.futex.clone()
            } else {
                // maybe should do clone here?
                Arc::new(Mutex::new(Futex::new()))
            },
            inner: Mutex::new(TaskControlBlockInner {
                // inherited
                pgid: parent_inner.pgid,
                heap_bottom: parent_inner.heap_bottom,
                heap_pt: parent_inner.heap_pt,
                // clone
                sigpending: parent_inner.sigpending.clone(),
                // new
                children: Vec::new(),
                rusage: Rusage::new(),
                clock: ProcClock::new(),
                clear_child_tid: 0,
                robust_list: RobustList::default(),
                timer: [ITimerVal::new(); 3],
                sigmask: Signals::empty(),
                // compute
                trap_cx_ppn,
                task_cx: TaskContext::goto_trap_return(kstack_top),
                parent: if flags.contains(CloneFlags::CLONE_PARENT)
                    | flags.contains(CloneFlags::CLONE_THREAD)
                {
                    parent_inner.parent.clone()
                } else {
                    Some(Arc::downgrade(self))
                },
                // constants
                task_status: TaskStatus::Ready,
                exit_code: 0,
            }),
        });
        // add child
        if flags.contains(CloneFlags::CLONE_PARENT) || flags.contains(CloneFlags::CLONE_THREAD) {
            if let Some(grandparent) = &parent_inner.parent {
                grandparent
                    .upgrade()
                    .unwrap()
                    .acquire_inner_lock()
                    .children
                    .push(task_control_block.clone());
            }
        } else {
            parent_inner.children.push(task_control_block.clone());
        }
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        if flags.contains(CloneFlags::CLONE_THREAD) {
            *trap_cx = *parent_inner.get_trap_cx();
        }
        // we also do not need to prepare parameters on stack, musl has done it for us
        if !stack.is_null() {
            trap_cx.gp.sp = stack as usize;
        }
        // set tp
        if flags.contains(CloneFlags::CLONE_SETTLS) {
            trap_cx.gp.tp = tls;
        }
        // for child process, fork returns 0
        trap_cx.gp.a0 = 0;
        // modify kernel_sp in trap_cx
        trap_cx.kernel_sp = kstack_top;
        // return
        task_control_block
        // ---- release parent PCB lock
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn setpgid(&self, pgid: usize) -> isize {
        if (pgid as isize) < 0 {
            return -1;
        }
        let mut inner = self.acquire_inner_lock();
        inner.pgid = pgid;
        0
        //Temporarily suspend. Because the type of 'self' is 'Arc', which can't be borrow as mutable.
    }
    pub fn getpgid(&self) -> usize {
        let inner = self.acquire_inner_lock();
        inner.pgid
    }
    pub fn get_user_token(&self) -> usize {
        self.vm.lock().token()
    }
}

impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        self.tid_allocator.lock().dealloc(self.tid);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
    Interruptible,
}
