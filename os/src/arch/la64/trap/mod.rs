mod context;
mod mem_access;
use self::context::GeneralRegs;

use super::register::{self, Exception, Interrupt, Trap, ERA};
use super::{pre_start_init, MErrEntry};
use crate::arch::la64::laflex::LAFlexPageTable;
use crate::arch::la64::register::{CrMd, ECfg, LineBasedInterrupt, PrMd, TCfg, TIClr};
use crate::arch::la64::trap::mem_access::Instruction;
use crate::arch::{get_clock_freq, TICKS_PER_SEC};
use crate::mm::{copy_from_user, copy_to_user, frame_reserve, MemoryError, PageTable, VirtAddr};
use crate::syscall::syscall;
use crate::task::{
    current_task, current_trap_cx, current_user_token, do_signal, do_wake_expired,
    suspend_current_and_run_next, Signals,
};
use core::arch::{asm, global_asm};
use core::ptr::{addr_of, addr_of_mut};

pub use context::{MachineContext, TrapContext, UserContext};
use register::{
    BadV, EStat, TLBRBadV, TLBREHi, TLBRELo0, TLBRELo1, TLBRPrMd, PGD, PGDH, PGDL, PWCH, PWCL,
    TLBRERA,
};
pub type TrapImpl = Trap;
//pub type InterruptImpl = Interrupt;
//pub type ExceptionImpl = Exception;
global_asm!(include_str!("trap.S"));

extern "C" {
    pub fn __alltraps();
    pub fn __restore();
    pub fn __call_sigreturn();
    pub fn strampoline();
    pub fn __kern_trap();
}

#[allow(unused, undefined_naked_function_abi)]
#[link_section = ".text.__rfill"]
#[naked]
#[no_mangle]
pub fn __rfill() {
    //crmd = 0b0_01_01_10_0_00;
    //         w_dm_df_pd_i_lv;
    // let i = 0xA8;
    unsafe {
        asm!(
            // PGD: 0x1b CRMD:0x0 PWCL:0x1c TLBRBADV:0x89 TLBERA:0x8a TLBRSAVE:0x8b SAVE:0x30
            // TLBREHi: 0x8e STLBPS: 0x1e MERRsave:0x95
            "
    csrwr  $t0, 0x8b



    csrrd  $t0, 0x1b
    lddir  $t0, $t0, 3
    andi   $t0, $t0, 1
    beqz   $t0, 1f

    csrrd  $t0, 0x1b
    lddir  $t0, $t0, 3
    addi.d $t0, $t0, -1
    lddir  $t0, $t0, 1
    andi   $t0, $t0, 1
    beqz   $t0, 1f
    csrrd  $t0, 0x1b
    lddir  $t0, $t0, 3
    addi.d $t0, $t0, -1
    lddir  $t0, $t0, 1
    addi.d $t0, $t0, -1

    ldpte  $t0, 0
    ldpte  $t0, 1
    csrrd  $t0, 0x8c
    csrrd  $t0, 0x8d
    csrrd  $t0, 0x0
2:
    tlbfill
    csrrd  $t0, 0x89
    srli.d $t0, $t0, 13
    slli.d $t0, $t0, 13
    csrwr  $t0, 0x11
    tlbsrch
    tlbrd
    csrrd  $t0, 0x12
    csrrd  $t0, 0x13
    csrrd  $t0, 0x8b
    ertn
1:
    csrrd  $t0, 0x8e
    ori    $t0, $t0, 0xC
    csrwr  $t0, 0x8e

    rotri.d $t0, $t0, 61
    ori    $t0, $t0, 3
    rotri.d $t0, $t0, 3

    csrwr  $t0, 0x8c
    csrrd  $t0, 0x8c
    csrwr  $t0, 0x8d
    b      2b
",
            options(noreturn)
        )
    }
}

pub fn init() {
    set_kernel_trap_entry();
}
pub fn get_bad_ins_addr() -> usize {
    match get_exception_cause() {
        Trap::Interrupt(_) | Trap::Exception(_) => register::ERA::read().get_pc(),
        Trap::TLBReFill => register::TLBRERA::read().get_pc(),
        Trap::MachineError(_) => register::MErrEra::read().get_pc(),
        Trap::Unknown => 0,
    }
}
pub fn get_bad_addr() -> usize {
    match get_exception_cause() {
        Trap::Exception(_) => register::BadV::read().get_vaddr(),
        Trap::TLBReFill => register::TLBRBadV::read().get_vaddr(),
        _ => 0,
    }
}
pub fn get_bad_instruction() -> usize {
    register::BadI::read().get_inst()
}
pub fn get_exception_cause() -> TrapImpl {
    register::EStat::read().cause()
}
pub fn set_kernel_trap_entry() {
    register::EEntry::read()
        .set_exception_entry(__kern_trap as usize)
        .write()
}
pub fn set_machine_err_trap_ent() {
    MErrEntry::read().set_addr(trap_handler as usize).write();
}

fn set_user_trap_entry() {
    register::EEntry::read()
        .set_exception_entry(strampoline as usize)
        .write();
}

pub fn enable_timer_interrupt() {
    let timer_freq = get_clock_freq();
    TCfg::read()
        .set_enable(true)
        .set_periodic(false)
        .set_init_val(timer_freq / TICKS_PER_SEC)
        .write();
    ECfg::empty()
        .set_line_based_interrupt_vector(LineBasedInterrupt::TIMER)
        .write();
}
#[link_section = ".text.trap_handler"]
#[no_mangle]
pub fn trap_handler() -> ! {
    if PrMd::read().get_pplv() == 0 {
        panic!();
    }
    set_kernel_trap_entry();

    {
        let task = current_task().unwrap();
        let mut inner = task.acquire_inner_lock();
        inner.update_process_times_enter_trap();
    }

    let cause = get_exception_cause();
    let stval = get_bad_addr();
    let badi = get_bad_instruction();
    log::debug!("[trap_handler]Cause:{:?}", cause);
    match cause {
        Trap::Exception(Exception::Syscall) => {
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            ERA::read().next_ins().write();
            cx.gp.pc += 4;
            // get system call return value
            let result = syscall(
                cx.gp.a7,
                [cx.gp.a0, cx.gp.a1, cx.gp.a2, cx.gp.a3, cx.gp.a4, cx.gp.a5],
            );
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.gp.a0 = result as usize;
        }
        Trap::Exception(Exception::PagePrivilegeIllegal)
        | Trap::Exception(Exception::PageInvalidFetch)
        | Trap::Exception(Exception::PageInvalidStore)
        | Trap::Exception(Exception::PageInvalidLoad)
        | Trap::Exception(Exception::PageModifyFault)
        | Trap::Exception(Exception::PageNonReadableFault)
        | Trap::Exception(Exception::PageNonExecutableFault) => {
            let task = current_task().unwrap();
            let mut inner = task.acquire_inner_lock();
            let addr = VirtAddr::from(get_bad_addr());
            log::debug!("[page_fault] pid: {}, type: {:?}", task.pid.0, cause);
            log::debug!(
                "[page_fault] {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                TLBRERA::read(),
                TLBRBadV::read(),
                TLBREHi::read(),
                TLBRELo0::read(),
                TLBRELo1::read(),
                PWCL::read(),
            );
            // This is where we handle the page fault.
            frame_reserve(3);
            let mut mset_lock = task.vm.lock();
            match mset_lock.do_page_fault(addr) {
                Err(error) => match error {
                    MemoryError::BeyondEOF => {
                        inner.add_signal(Signals::SIGBUS);
                    }
                    MemoryError::NoPermission | MemoryError::BadAddress => {
                        inner.add_signal(Signals::SIGSEGV);
                    }
                    _ => unreachable!(),
                },
                Ok(_) => {
                    //tlb_addr_allow_write(addr.floor(), _paddr.floor()).unwrap();
                    drop(mset_lock);
                    if let Trap::Exception(
                        Exception::PageModifyFault | Exception::PageInvalidStore,
                    ) = cause
                    {
                        LAFlexPageTable::from_token(task.get_user_token())
                            .set_dirty_bit(addr.floor())
                            .unwrap();
                    }
                }
            };
        }
        Trap::Exception(Exception::InstructionNonDefined)
        | Trap::Exception(Exception::InstructionPrivilegeIllegal) => {
            let task = current_task().unwrap();
            let mut inner = task.acquire_inner_lock();
            inner.add_signal(Signals::SIGILL);
        }
        Trap::Interrupt(Interrupt::Timer) => {
            do_wake_expired();
            TIClr::read().clear_timer().write();
            enable_timer_interrupt();
            suspend_current_and_run_next();
        }
        Trap::Exception(Exception::Breakpoint) => {
            read_bp();
        }
        Trap::Exception(Exception::AddressNotAligned) => {
            let cx = current_trap_cx();
            let token = current_user_token();
            let pc = cx.gp.pc;
            let mut i = 0;
            copy_from_user(token, pc as *const u32, addr_of_mut!(i)).unwrap();
            let ins = Instruction::from(i);
            let op = ins.get_op_code();
            if op.is_err() {
                panic!("Unsupported OpCode! Instruction: {:?} ", ins);
            }
            let op = op.unwrap();
            let addr = BadV::read().get_vaddr();
            //debug!("{:#x}: {:?}, {:#x}", pc, op, addr);
            let sz = op.get_size();
            let is_aligned: bool = addr % sz == 0;
            if !is_aligned {
                assert!([2, 4, 8].contains(&sz));
                if op.is_store() {
                    let mut rd = if !op.is_float_op() {
                        cx.gp[ins.get_rd_num()]
                    } else {
                        cx.fp.f[ins.get_rd_num()]
                    };
                    for i in 0..sz {
                        let seg = rd as u8;
                        copy_to_user(token, addr_of!(seg), (addr + i) as *mut u8).unwrap();
                        rd >>= 8;
                    }
                } else {
                    let mut rd = 0;
                    for i in (0..sz).rev() {
                        rd <<= 8;
                        let mut read_byte: u8 = 0;
                        copy_from_user(token, (i + addr) as *const u8, addr_of_mut!((read_byte)))
                            .unwrap();
                        rd |= read_byte as usize;
                    }
                    if !op.is_unsigned_ld() {
                        match sz {
                            2 => rd = (rd as u16) as i16 as isize as usize,
                            4 => rd = (rd as u32) as i32 as isize as usize,
                            8 => rd = rd,
                            _ => unreachable!(),
                        }
                    }
                    if !op.is_float_op() {
                        cx.gp[ins.get_rd_num()] = rd;
                    } else {
                        cx.fp.f[ins.get_rd_num()] = rd;
                    }
                }
                cx.gp.pc += 4;
            }
            if cx.gp.pc == pc {
                panic!(
                    "Failed to execute the command. Bad Instruction: {}, PC:{}",
                    unsafe { *(cx.gp.pc as *const u32) },
                    pc
                );
            }
        }
        Trap::Interrupt(Interrupt::IPI)
        | Trap::MachineError(_)
        | Trap::Unknown
        | Trap::Exception(Exception::AddressError)
        | _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}, BadI = {:#x}!",
                cause, stval, badi
            );
        }
    }
    {
        let task = current_task().unwrap();
        let mut inner = task.acquire_inner_lock();
        inner.update_process_times_leave_trap(cause);
    }
    trap_return();
}

fn read_bp() {
    println!(
        "[trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}",
        PrMd::read(),
        TLBRERA::read(),
        TLBRBadV::read(),
        TLBRPrMd::read(),
        TLBREHi::read(),
        TLBRELo0::read(),
        TLBRELo1::read(),
        PGD::read(),
        PWCL::read(),
        PWCH::read()
    );
    let cause = get_exception_cause();
    let stval = get_bad_addr();
    let badi = get_bad_instruction();
    panic!(
        "[trap_handler] {:?}, stval = {:#x}, BadI = {:#x}!",
        cause, stval, badi
    );
}
#[no_mangle]
pub fn trap_return() -> ! {
    do_signal();
    set_user_trap_entry();
    let task = current_task().unwrap();
    let trap_cx = task.acquire_inner_lock().get_trap_cx();
    let trap_cx_ptr = trap_cx as *const TrapContext as usize;
    trap_cx.sstatus.set_pplv(3).set_pie(true);
    //log::debug!("[trap_return] trap_cx:{:?}", trap_cx);
    let user_satp = task.get_user_token();
    //log::debug!("[trap_return] trap_cx_ptr:{:#x}, user_satp:{:#x}", trap_cx_ptr, user_satp);
    drop(task);
    let restore_va = __restore as usize - __alltraps as usize + strampoline as usize;
    pre_start_init();
    unsafe {
        asm!(
            "ibar 0",
            "move $ra, {0}",
            "move $a0, {1}",
            "move $a1, {2}",
            "jr $ra",
            in(reg) restore_va,
            in(reg) trap_cx_ptr,
            in(reg) user_satp,
            options(noreturn)
        );
    }
}

/// The KERNEL SPACE trap handler
/// # ERA
/// The ERA kept "as-is" in the `__kern_trap` (See `trap.S`) after this function call.
/// If modification to `ERA` is needed, this should be taken into account.
#[no_mangle]
pub extern "C" fn trap_from_kernel(gr: &mut GeneralRegs) {
    let cause = get_exception_cause();
    let sub_code = EStat::read().exception_sub_code();
    match cause {
        Trap::TLBReFill => {
            println!(
                "[trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}\n\
                 [trap_handler] {:?}",
                CrMd::read(),
                TLBRERA::read(),
                TLBRBadV::read(),
                TLBRPrMd::read(),
                PGD::read(),
                PWCL::read(),
                PWCH::read()
            );
        }
        Trap::Exception(Exception::AddressNotAligned) => {
            let pc = gr.pc;
            loop {
                let ins = Instruction::from(gr.pc as *const Instruction);
                let op = ins.get_op_code();
                if op.is_err() {
                    break;
                }
                let op = op.unwrap();
                let addr = BadV::read().get_vaddr();
                //debug!("{:#x}: {:?}, {:#x}", pc, op, addr);
                let sz = op.get_size();
                let is_aligned: bool = addr % sz == 0;
                if is_aligned {
                    break;
                }
                assert!([2, 4, 8].contains(&sz));
                if op.is_store() {
                    let mut rd = gr[ins.get_rd_num()];
                    for i in 0..sz {
                        unsafe { ((addr + i) as *mut u8).write_unaligned(rd as u8) };
                        rd >>= 8;
                    }
                } else {
                    let mut rd = 0;
                    for i in (0..sz).rev() {
                        rd <<= 8;
                        let read_byte =
                            (unsafe { ((addr + i) as *mut u8).read_unaligned() } as usize);
                        rd |= read_byte;
                        //debug!("{:#x}, {:#x}", rd, read_byte);
                    }
                    if !op.is_unsigned_ld() {
                        match sz {
                            2 => rd = (rd as u16) as i16 as isize as usize,
                            4 => rd = (rd as u32) as i32 as isize as usize,
                            8 => rd = rd,
                            _ => unreachable!(),
                        }
                    }
                    gr[ins.get_rd_num()] = rd;
                }
                gr.pc += 4;
                break;
            }
            if gr.pc == pc {
                panic!(
                    "Failed to execute the command. Bad Instruction: {}, PC:{}",
                    unsafe { *(gr.pc as *const u32) },
                    pc
                );
            }
            //debug!("{:?}", gr);
            return;
        }
        _ => {}
    }
    panic!(
        "a trap {:?} from kernel! bad addr = {:#x}, bad instruction = {:#x}, pc:{:#x}, (subcode:{}), PGDH: {:?}, PGDL: {:?}, {}",
        cause,
        get_bad_addr(),
        get_bad_instruction(),
        get_bad_ins_addr(),
        sub_code,
        PGDH::read(),
        PGDL::read(),
        if let Trap::Exception(ty) = cause {
            match ty {
                Exception::AddressError => match sub_code {
                    0 => "ADdress error Exception for Fetching instructions",
                    1 => "ADdress error Exception for Memory access instructions",
                    _ => "Unknown",
                },
                _ => "",
            }
        } else {
            ""
        }
    );
}
