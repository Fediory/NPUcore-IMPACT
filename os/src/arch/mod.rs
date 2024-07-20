#[cfg(feature = "la64")]
mod la64;
#[cfg(feature = "la64")]
pub use la64::{
    board,
    board::BlockDeviceImpl,
    board::MMIO,
    bootstrap_init, config,
    config::BUFFER_CACHE_NUM,
    console_flush, console_getchar, console_putchar, machine_init, shutdown,
    time::{get_clock_freq, get_time, TICKS_PER_SEC},
    KernelPageTableImpl, PageTableImpl, __switch, syscall_id, tlb_invalidate,
    trap::{
        get_bad_addr, get_bad_instruction, get_exception_cause, trap_handler, trap_return,
        MachineContext, TrapContext, TrapImpl, UserContext,
    },
    trap_cx_bottom_from_tid, ustack_bottom_from_tid, KernelStack, BLOCK_SZ,
};
