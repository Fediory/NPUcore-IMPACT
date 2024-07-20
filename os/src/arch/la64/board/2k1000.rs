use crate::drivers::block::sata_blk::SataBlock;
pub const MMIO: &[(usize, usize)] = &[];
use crate::config::HIGH_BASE_EIGHT;
pub type BlockDeviceImpl = SataBlock;

pub const BLOCK_SZ: usize = 2048;
pub const UART_BASE: usize = 0x1fe2_0000 + HIGH_BASE_EIGHT;
pub const ACPI_BASE: usize = 0x1fe2_7000 + HIGH_BASE_EIGHT;
