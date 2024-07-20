mod context;
mod elf;
mod manager;
mod pid;
mod processor;
pub mod signal;
mod task;
pub mod threads;

use crate::arch::__switch;
use crate::{
    fs::{OpenFlags, ROOT_FD},
    mm::translated_refmut,
};
use alloc::{collections::VecDeque, sync::Arc};
pub use context::TaskContext;
pub use elf::{load_elf_interp, AuxvEntry, AuxvType, ELFInfo};
use lazy_static::*;
use manager::fetch_task;
pub use manager::{
    add_task, do_oom, do_wake_expired, find_task_by_pid, find_task_by_tgid, procs_count,
    sleep_interruptible, wait_with_timeout, wake_interruptible,
};
pub use pid::{
    pid_alloc, trap_cx_bottom_from_tid, ustack_bottom_from_tid, KernelStackImpl, PidHandle,
};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
};
pub use signal::*;
pub use task::{RobustList, Rusage, TaskControlBlock, TaskStatus};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- hold current PCB lock
    let mut task_inner = task.acquire_inner_lock();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB lock

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn block_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- hold current PCB lock
    let mut task_inner = task.acquire_inner_lock();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Interruptible
    task_inner.task_status = TaskStatus::Interruptible;
    drop(task_inner);
    // ---- release current PCB lock

    // push to interruptible queue of scheduler, so that it won't be scheduled.
    sleep_interruptible(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn do_exit(task: Arc<TaskControlBlock>, exit_code: u32) {
    // **** hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    if !task.exit_signal.is_empty() {
        let parent_task = inner.parent.as_ref().unwrap().upgrade().unwrap(); // this will acquire inner of current task
        let mut parent_inner = parent_task.acquire_inner_lock();
        parent_inner.add_signal(task.exit_signal);

        if parent_inner.task_status == TaskStatus::Interruptible {
            // wake up parent if parent is waiting.
            parent_inner.task_status = TaskStatus::Ready;
            drop(parent_inner);
            // push back to ready queue.
            wake_interruptible(parent_task);
        }
    }
    log::trace!(
        "[do_exit] Trying to exit pid {} with {}",
        task.pid.0,
        exit_code
    );
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;

    // move children to initproc
    if !inner.children.is_empty() {
        let mut initproc_inner = INITPROC.acquire_inner_lock();
        while let Some(child) = inner.children.pop() {
            child.acquire_inner_lock().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child);
        }
        if initproc_inner.task_status == TaskStatus::Interruptible {
            // wake up initproc if initproc is waiting.
            initproc_inner.task_status = TaskStatus::Ready;
            // push back to ready queue.
            wake_interruptible(INITPROC.clone());
        }
    }

    inner.children.clear();
    if inner.clear_child_tid != 0 {
        log::debug!(
            "[do_exit] do futex wake on clear_child_tid: {:X}",
            inner.clear_child_tid
        );
        //let phys_ref =
        match translated_refmut(task.get_user_token(), inner.clear_child_tid as *mut u32) {
            Ok(phys_ref) => {
                *phys_ref = 0;
                task.futex.lock().wake(phys_ref as *const u32 as usize, 1);
            }
            Err(_) => log::warn!("invalid clear_child_tid"),
        };
    }
    // deallocate user resource (trap context and user stack)
    task.vm.lock().dealloc_user_res(task.tid);
    // deallocate whole user space in advance, or if its parent do not call wait,
    // this resource may not be recycled in a long period of time.
    if Arc::strong_count(&task.vm) == 1 {
        task.vm.lock().recycle_data_pages();
    }
    drop(inner);
    // **** release current PCB lock
    // drop task manually to maintain rc correctly
    log::info!("[do_exit] Pid {} exited with {}", task.pid.0, exit_code);
}

pub fn exit_current_and_run_next(exit_code: u32) -> ! {
    // take from Processor
    let task = take_current_task().unwrap();
    do_exit(task, exit_code);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
    panic!("Unreachable");
}

pub fn exit_group_and_run_next(exit_code: u32) -> ! {
    // exit current, take from Processor
    let task = take_current_task().unwrap();
    let tgid = task.tgid;
    do_exit(task, exit_code);

    let mut exit_list = VecDeque::new();

    let mut manager = manager::TASK_MANAGER.lock();
    let mut remain = manager.ready_queue.len();
    while let Some(task) = manager.ready_queue.pop_front() {
        if task.tgid == tgid {
            exit_list.push_back(task);
        } else {
            manager.ready_queue.push_back(task);
        }
        remain -= 1;
        if remain == 0 {
            break;
        }
    }
    let mut remain = manager.interruptible_queue.len();
    while let Some(task) = manager.interruptible_queue.pop_front() {
        if task.tgid == tgid {
            exit_list.push_back(task);
        } else {
            manager.interruptible_queue.push_back(task);
        }
        remain -= 1;
        if remain == 0 {
            break;
        }
    }
    drop(manager);

    for task in exit_list.into_iter() {
        do_exit(task, exit_code);
    }
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
    panic!("Unreachable");
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let elf = ROOT_FD.open("initproc", OpenFlags::O_RDONLY, true).unwrap();
        TaskControlBlock::new(elf)
    });
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}
