
use crate::drivers::block::MemBlockWrapper;
pub const MMIO: &[(usize, usize)] = &[];

pub type BlockDeviceImpl = MemBlockWrapper;



pub const DMA_ADDR: usize = 0x1fe0_0c00 | HIGH_BASE_EIGHT;
pub const ROOT_BASE_ADDR: usize = 0x00e0_0000;
pub const BLOCK_SZ: usize = 2048;
pub const UART_BASE: usize = 0x1fe2_0000;