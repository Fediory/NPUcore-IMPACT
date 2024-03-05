// SPDX-License-Identifier: GPL-2.0
// All sources in the current directory("The Code") is a reimplementation of Loongson's NAND and DMA drivers,
// which had GPL-2.0 as its license. The code therefore inherits and adapt it as its license.
use self::dma::DMAOrder;
use crate::{
    arch::{board, BLOCK_SZ},
    fs::BlockDevice,
};

use spin::Mutex;

use super::board::{DMA_ADDR, NAND_BASE, ROOT_BASE_ADDR};
#[macro_use]
mod mmio_macro;
mod dma;
mod nand;
pub use nand::{Nand, NandType};
pub struct LoongsonNand {
    nand: Nand,
    dma: Mutex<DMAOrder>,
    start_page: usize,
}
impl LoongsonNand {
    pub fn new() -> Self {
        Self::do_new(NAND_BASE, 0, DMA_ADDR, ROOT_BASE_ADDR / BLOCK_SZ)
    }
    fn do_new(nand_base: usize, chip: usize, dma_base: usize, start_page: usize) -> Self {
        assert!(chip < 4);
        let mut nand = Nand::new(nand_base, chip);
        //nand.hw_init();
        nand.reset();
        Self {
            nand,
            dma: Mutex::new(DMAOrder::empty().set_value(dma_base as u32)),
            start_page,
        }
    }
}

impl BlockDevice for LoongsonNand {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        assert!(buf.len() % BLOCK_SZ == 0);
        let mut lock = self.dma.lock();
        for i in 0..buf.len() / BLOCK_SZ {
            self.nand
                .read(
                    &mut lock,
                    &mut buf[i * BLOCK_SZ..(1 + i) * BLOCK_SZ],
                    block_id + self.start_page + i,
                    false,
                )
                .unwrap();
        }
        drop(lock);
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        /* assert!(buf.len() % BLOCK_SZ == 0); */
        //self.clear_mult_block(block_id, buf.len() / BLOCK_SZ, 0);
        /* let mut lock = self.dma.lock();
         * for i in 0..buf.len() / BLOCK_SZ {
         *     self.nand
         *         .page_program(
         *             &mut lock,
         *             &buf[i * BLOCK_SZ..(1 + i) * BLOCK_SZ],
         *             block_id + self.start_page + i,
         *         )
         *         .unwrap();
         * }
         * drop(lock); */
    }

    #[inline(always)]
    fn clear_block(&self, block_id: usize, num: u8) {
        if num != 0 {
            self.write_block(block_id, &[num; board::BLOCK_SZ]);
        } else {
            self.nand.erase(block_id, 1);
        }
    }
    #[inline(always)]
    fn clear_mult_block(&self, block_id: usize, cnt: usize, num: u8) {
        if num != 0 {
            for i in block_id..block_id + cnt {
                self.write_block(i, &[num; board::BLOCK_SZ]);
            }
        } else {
            self.nand.erase(block_id, cnt)
        }
    }
}
