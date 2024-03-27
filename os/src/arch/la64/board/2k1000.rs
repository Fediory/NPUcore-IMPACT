
use crate::drivers::block::MemBlockWrapper;
pub const MMIO: &[(usize, usize)] = &[];

pub type BlockDeviceImpl = MemBlockWrapper;

pub const BLOCK_SZ: usize = 2048;
pub const UART_BASE: usize = 0x1fe2_0000;