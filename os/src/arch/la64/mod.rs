#[cfg(feature = "board_2k1000")]
#[path = "board/2k1000.rs"]
pub mod board;
#[cfg(feature = "board_laqemu")]
#[path = "board/2k1000.rs"]
pub mod board;
pub mod config;
pub mod laflex;
mod sbi;
pub mod switch;
pub mod time;
pub mod trap;
pub type KernelPageTableImpl = laflex::LAFlexPageTable;
pub type PageTableImpl = laflex::LAFlexPageTable;
pub use sbi::{console_flush, console_getchar, console_putchar, shutdown};
pub use switch::__switch;
pub use tlb::{tlb_global_invalidate, tlb_invalidate};
pub mod syscall_id;

//use crate::mm::remap_test;

use crate::{
    arch::la64::{
        board::UART_BASE,
        trap::{set_kernel_trap_entry, set_machine_err_trap_ent},
    },
    config::{DIR_WIDTH, MMAP_BASE, PAGE_SIZE_BITS, PTE_WIDTH, PTE_WIDTH_BITS, SUC_DMW_VESG},
};

use self::{time::get_timer_freq_first_time, trap::strampoline};
pub use board::BLOCK_SZ;
pub use kern_stack::{trap_cx_bottom_from_tid, ustack_bottom_from_tid, KernelStack};
pub use register::*;
mod kern_stack;
mod la_libc_import;
mod register;
mod tlb;
extern "C" {
    pub fn srfill();
}
pub fn machine_init() {
    // remap_test not supported for lack of DMW read only privilege support
    trap::init();
    get_timer_freq_first_time();
    /* println!(
     *     "[machine_init] VALEN: {}, PALEN: {}",
     *     cfg0.get_valen(),
     *     cfg0.get_palen()
     * ); */
    for i in 0..=6 {
        let j: usize;
        unsafe { core::arch::asm!("cpucfg {0},{1}",out(reg) j,in(reg) i) };
        println!("[CPUCFG {:#x}] {}", i, j);
    }
    for i in 0x10..=0x14 {
        let j: usize;
        unsafe { core::arch::asm!("cpucfg {0},{1}",out(reg) j,in(reg) i) };
        println!("[CPUCFG {:#x}] {}", i, j);
    }
    println!("{:?}", Misc::read());
    println!("{:?}", RVACfg::read());
    println!("[machine_init] MMAP_BASE: {:#x}", MMAP_BASE);
    trap::enable_timer_interrupt();
}
pub fn pre_start_init() {
    EEntry::empty().set_exception_entry(strampoline as usize);
}
pub fn bootstrap_init() {
    /* if CPUId::read().get_core_id() != 0 {
     *     loop {}
     * } */
    ECfg::empty()
        .set_line_based_interrupt_vector(LineBasedInterrupt::TIMER)
        .write();
    EUEn::read().set_float_point_stat(true).write();
    // Timer & other Interrupts
    TIClr::read().clear_timer().write();
    TCfg::read().set_enable(false).write();
    CrMd::read()
        .set_watchpoint_enabled(false)
        .set_paging(true)
        .set_ie(false)
        .write();

    // Trap/Exception Hanlder initialization.
    set_kernel_trap_entry();
    set_machine_err_trap_ent();
    TLBREntry::read().set_addr(srfill as usize).write();

    // MMU Setup
    DMW2::read()
        .set_plv0(true)
        .set_plv1(false)
        .set_plv2(false)
        .set_plv3(false)
        .set_vesg(SUC_DMW_VESG)
        .set_mat(MemoryAccessType::StronglyOrderedUnCached)
        .write();
    DMW3::empty().write();
    //DMW1::empty().write();

    STLBPS::read().set_ps(PTE_WIDTH_BITS).write();
    TLBREHi::read().set_page_size(PTE_WIDTH_BITS).write();
    PWCL::read()
        .set_ptbase(PAGE_SIZE_BITS)
        .set_ptwidth(DIR_WIDTH)
        .set_dir1_base(PAGE_SIZE_BITS + DIR_WIDTH)
        .set_dir1_width(DIR_WIDTH) // 512*512*4096 should be enough for 256MiB of 2k1000.
        .set_dir2_base(0)
        .set_dir2_width(0)
        .set_pte_width(PTE_WIDTH)
        .write();
    PWCH::read()
        .set_dir3_base(PAGE_SIZE_BITS + DIR_WIDTH * 2)
        .set_dir3_width(DIR_WIDTH)
        .set_dir4_base(0)
        .set_dir4_width(0)
        .write();

    println!("[kernel] UART address: {:#x}", UART_BASE);
    println!("[bootstrap_init] {:?}", PRCfg1::read());
}
