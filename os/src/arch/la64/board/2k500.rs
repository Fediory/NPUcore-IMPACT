use crate::config::HIGH_BASE_EIGHT;
//use core::ptr::write_volatile;

use super::ls_nand::{self, NandType};
pub const MMIO: &[(usize, usize)] = &[];

pub type BlockDeviceImpl = ls_nand::LoongsonNand;
/* use crate::drivers::block::MemBlockWrapper;
 * pub type BlockDeviceImpl = MemBlockWrapper; */
pub const NAND_TYPE: NandType = NandType::Sz2GbPg2K;
pub const NAND_BASE: usize = 0x1ff5_8000 | HIGH_BASE_EIGHT;
pub const DMA_ADDR: usize = 0x1fe1_0c00 | HIGH_BASE_EIGHT;
pub const ROOT_BASE_ADDR: usize = 0x00e0_0000;
pub const BLOCK_SZ: usize = 2048;
pub const UART_MUX: usize = 2;
pub const UART_BASE: usize = 0x0_000_0000_1ff4_0000 + HIGH_BASE_EIGHT + {
    assert!(UART_MUX < 10);
    0x400 * UART_MUX
};
