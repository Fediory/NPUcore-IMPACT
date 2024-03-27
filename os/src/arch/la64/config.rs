// Sizes
pub const MEMORY_SIZE: usize = 0x1000_0000;
pub const USER_STACK_SIZE: usize = PAGE_SIZE * 40;
pub const USER_HEAP_SIZE: usize = PAGE_SIZE * 20;
pub const SYSTEM_TASK_LIMIT: usize = 128;
pub const SYSTEM_FD_LIMIT: usize = 256;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = PAGE_SIZE.trailing_zeros() as usize;
pub const PTE_WIDTH: usize = 8;
pub const PTE_WIDTH_BITS: usize = PTE_WIDTH.trailing_zeros() as usize;
pub const DIR_WIDTH: usize = PAGE_SIZE_BITS - PTE_WIDTH_BITS;

#[cfg(debug_assertions)]
pub const KSTACK_PG_NUM_SHIFT: usize = 16usize.trailing_zeros() as usize;
#[cfg(not(debug_assertions))]
pub const KSTACK_PG_NUM_SHIFT: usize = 2usize.trailing_zeros() as usize;

pub const KERNEL_STACK_SIZE: usize = PAGE_SIZE << KSTACK_PG_NUM_SHIFT;
pub const KERNEL_HEAP_SIZE: usize = PAGE_SIZE * 0x3000;

// Addresses
/// Maximum length of a physical address
pub const PALEN: usize = 48;
/// Maximum length of a virtual address
pub const VALEN: usize = 48;
/// Maximum address in virtual address space.
/// May be used to extract virtual address from a segmented address
/// `0`-extension may be performed using this mask.
/// e.g. `flag` &= `VA_MASK`
pub const VA_MASK: usize = (1 << VALEN) - 1;
/// Mask for extracting segment number from usize address.
/// `1`-extension may be performed using this mask.
/// e.g. `flag` |= `SEG_MASK`
pub const SEG_MASK: usize = !VA_MASK;
/// Mask for extracting segment number from VPN.
/// All-one for segment field.
/// `1`-extension may be performed using this mask.
/// e.g. `flag` |= `SEG_MASK`
pub const VPN_SEG_MASK: usize = SEG_MASK >> PAGE_SIZE_BITS;

pub const HIGH_BASE_ZERO: usize = 0x0000_0000_0000_0000;

// manually make usable memory space equal
pub const SUC_DMW_VESG: usize = 8;
pub const MEMORY_HIGH_BASE: usize = HIGH_BASE_ZERO;
pub const MEMORY_HIGH_BASE_VPN: usize = MEMORY_HIGH_BASE >> PAGE_SIZE_BITS;
pub const USER_STACK_BASE: usize = TASK_SIZE - PAGE_SIZE | LA_START;
pub const MEMORY_START: usize = 0x0000_0000_9000_0000;
pub const MEMORY_END: usize = MEMORY_SIZE + MEMORY_START;

pub const SV39_SPACE: usize = 1 << 39;
pub const USR_SPACE_LEN: usize = SV39_SPACE >> 2;
pub const LA_START: usize = 0x1_2000_0000;
pub const USR_VIRT_SPACE_END: usize = USR_SPACE_LEN - 1;
pub const TRAMPOLINE: usize = SIGNAL_TRAMPOLINE; // The trampoline is NOT mapped in LA.
pub const SIGNAL_TRAMPOLINE: usize = USR_VIRT_SPACE_END - PAGE_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = SIGNAL_TRAMPOLINE - PAGE_SIZE;
pub const USR_MMAP_END: usize = TRAP_CONTEXT_BASE - PAGE_SIZE;
pub const USR_MMAP_BASE: usize = USR_MMAP_END - USR_SPACE_LEN / 8 + 0x3000;
pub const TASK_SIZE: usize = USR_MMAP_BASE - USR_SPACE_LEN / 8;
pub const ELF_DYN_BASE: usize = (((TASK_SIZE - LA_START) / 3 * 2) | LA_START) & (!(PAGE_SIZE - 1));

pub const MMAP_BASE: usize = 0xFFFF_FF80_0000_0000;
pub const MMAP_END: usize = 0xFFFF_FFFF_FFFF_0000;
pub const SKIP_NUM: usize = 1;

pub const DISK_IMAGE_BASE: usize = 0x800_0000 + MEMORY_START;
pub const BUFFER_CACHE_NUM: usize = 256 * 1024 * 1024 / 2048 * 4 / 2048;

pub static mut CLOCK_FREQ: usize = 0;

use core::arch::asm;

#[macro_export]
macro_rules! signal_type {
    () => {
        u128
    };
}
#[macro_export]
macro_rules! def_cpu_cfg {
    ($name:ident, $num: literal) => {
        pub struct $name {
            bits: u32,
        }

        impl $name {
            // 读取index对应字的内容
            pub fn read() -> Self {
                let mut bits;
                bits = $num;
                unsafe {
                    asm!("cpucfg {},{}",out(reg) bits,in(reg) bits);
                }
                Self { bits }
            }
            pub fn get_bit(&self, index: usize) -> bool {
                bit_field::BitField::get_bit(&self.bits, index)
            }
            pub fn get_bits(&self, start: usize, end: usize) -> u32 {
                bit_field::BitField::get_bits(&self.bits, start..=end)
            }
        }
    };
}
def_cpu_cfg!(CPUCfg0, 0);
def_cpu_cfg!(CPUCfg4, 4);
def_cpu_cfg!(CPUCfg5, 5);
impl CPUCfg0 {
    pub fn get_valen(&self) -> usize {
        (self.get_bits(12, 19) + 1) as usize
    }
    pub fn get_palen(&self) -> usize {
        (self.get_bits(4, 11) + 1) as usize
    }
}
#[macro_export]
macro_rules! newline {
    () => {
        "\r\n"
    };
}

#[macro_export]
macro_rules! should_map_trampoline {
    () => {
        false
    };
}

#[macro_export]
macro_rules! read_tot_sec16 {
    ($name:expr) => {{
        /// *KEEP IT THIS WAY!*
        /// Some arch relies on this for their compilers implement misaligned read so wrongly.
        #[inline(never)]
        fn misaligned_rd(super_block: &BPB) -> u16 {
            let ret: u16;
            unsafe {
                core::arch::asm!(
                    "
ld.bu   $a1, $a0, 0x14
ld.bu   $a0, $a0, 0x13
slli.d  $a1, $a1, 0x8
or      $a0, $a1, $a0
",
                    in("$a0") super_block,
                    lateout("$a0") ret
                )
            };
            ret
        }
        misaligned_rd($name)
    }};
}

#[macro_export]
macro_rules! read_root_ent_cnt {
    ($name:expr) => {{
        /// *KEEP IT THIS WAY!*
        /// Some arch relies on this for their compilers implement misaligned read so wrongly.
        #[inline(never)]
        fn misaligned_rd(super_block: &BPB) -> u16 {
            let ret: u16;
            unsafe {
                core::arch::asm!(
                    "
ld.bu   $a1, $a0, 0x12
ld.bu   $a0, $a0, 0x11
slli.d  $a1, $a1, 0x8
or      $a0, $a1, $a0
",
                    in("$a0") super_block,
                    lateout("$a0") ret
                )
            };
            ret
        }
        misaligned_rd($name)
    }};
}

#[macro_export]
macro_rules! read_byts_per_sec {
    ($name:expr) => {{
        /// *KEEP IT THIS WAY!*
        /// Some arch relies on this for their compilers implement misaligned read so wrongly.
        #[inline(never)]
        fn misaligned_rd(super_block: &BPB) -> u16 {
            let ret: u16;
            unsafe {
                core::arch::asm!(
                    "
ld.bu   $a1, $a0, 0xc
ld.bu   $a0, $a0, 0xb
slli.d  $a1, $a1, 0x8
or      $a0, $a1, $a0
",
                    in("$a0") super_block,
                    lateout("$a0") ret
                )
            };
            ret
        }
        misaligned_rd($name)
    }};
}

#[macro_export]
macro_rules! misaligned_wr {
    ($name:expr,$val:expr) => {};
}

#[macro_export]
macro_rules! copy_from_name1 {
    ($dst:expr,$name1:expr) => {{
        let mut dst = unsafe { core::ptr::addr_of!($dst[0]) as usize };
        let mut src = unsafe { core::ptr::addr_of!($name1) as usize };
        let mut x = 0;
        // First of all, the increment should be placed after the access.
        for _ in 0..10 {
            unsafe {
                *((dst) as *mut u8) = *((src) as *const u8);
            }
            dst += 1;
            src += 1;
        }
    }};
}

#[macro_export]
macro_rules! copy_to_name1 {
    ($name1:expr,$src:expr) => {{
        let k: [u16; 5] = $src;
        let mut dst = unsafe { core::ptr::addr_of!($name1) as usize };
        let mut src = unsafe { core::ptr::addr_of!(k) as usize };
        // First of all, the increment should be placed after the access.
        for _ in 0..10 {
            unsafe {
                *((dst) as *mut u8) = *((src) as *const u8);
            }
            dst += 1;
            src += 1;
        }
    }};
}
