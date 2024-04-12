#![no_std]
#![feature(linkage)]
#![feature(asm_experimental_arch)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(global_asm)]
#![feature(lang_items)]
#[allow(dead_code)]
#[macro_use]
pub mod console;
#[cfg(target_arch = "loongarch64")]
mod la_libc_import;
mod lang_items;
mod syscall;
mod usr_call;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
pub use core::arch::global_asm;
pub use usr_call::*;

const USER_HEAP_SIZE: usize = 32768;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

//#[cfg(target_arch = "riscv64")]
#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[cfg(target_arch = "loongarch64")]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[linkage = "weak"]
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    #[cfg(target_arch = "loongarch64")]
    {
        use core::arch::asm;
        let argc: usize;
        let argv: usize;
        unsafe {
            asm!(
                "ld.d $a0, $sp, 16",
                "addi.d $a1, $sp, 24",
                out("$a0") argc,
                out("$a1") argv
            );
        }
        _parameter(argc, argv);
    }
    unreachable!();
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn _parameter(argc: usize, argv: usize) -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start =
            unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
        let len = (0usize..)
            .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    exit(main(argc, v.as_slice()));
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 6;
        const TRUNC = 1 << 9;
    }
}
