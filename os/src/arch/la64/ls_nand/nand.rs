#[allow(unused)]
use bit_field::BitField;
use core::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
};
//use log::debug;

use crate::{arch::board::NAND_TYPE, task::try_yield};

use super::dma::DMAOrder;
const NAND_DMA_BUFFER: usize = 0x40;
const NAND_CMD: usize = 4 * 0;
const ADDR_C: usize = 4 * 1;
const ADDR_R: usize = 4 * 2;
const NAND_TIMING: usize = 4 * 3;
const ID_L: usize = 4 * 4;
const STATUS_AND_ID_H: usize = 4 * 5;
const NAND_PARAM: usize = 4 * 6;
const NAND_OP_NUM: usize = 4 * 7;
const CS_RDY_MAP: usize = 4 * 8;
const DMA_ACC_ADDR: usize = 4 * 16;

impl_define_mem_reg!(NandCmd, NAND_CMD, "NAND CMD register");
impl_define_mem_reg!(AddrC, ADDR_C, "In-page Offset register");
impl_define_mem_reg!(AddrR, ADDR_R, "register");
impl_define_mem_reg!(NandTiming, NAND_TIMING, "Timing register");
impl_define_csr_rd_only!(IdL, ID_L, "Register to store the lower bits of ID");

impl_define_csr_rd_only!(
    StatusAndIdH,
    STATUS_AND_ID_H,
    "Status and ID Higher bits register"
);
impl_define_mem_reg!(NandParam, NAND_PARAM, "Parameter register");
impl_define_mem_reg!(NandOpNum, NAND_OP_NUM, "Operator number register");
impl_define_mem_reg!(CsRdyMap, CS_RDY_MAP, "register");
impl_define_mem_reg!(DMAAccAddr, DMA_ACC_ADDR, "register");
impl StatusAndIdH {
    pub fn get_status(&self) -> usize {
        self.bits.get_bits(16..=23) as usize
    }
}
impl Debug for NandCmd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NandCmd")
            .field("status", &self.bits.get_bits(25..=29))
            .field("cs", &self.bits.get_bits(20..=23))
            .field("rdy", &self.bits.get_bits(16..=19))
            .field("command_valid", &self.is_cmd_valid())
            .field("erase_op", &self.is_erase_op())
            .field("wr_op", &self.is_write_op())
            .field("rd_op", &self.is_read_op())
            .field("done", &self.is_done())
            .field("main area", &self.is_main())
            .field("spare area", &self.is_spare())
            .field("ecc dma", &self.is_ecc_dma_req())
            .field("normal dma", &self.is_dma_req())
            .field("rd_stat", &self.get_read_status())
            .finish()
    }
}
impl Debug for NandParam {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NandParam")
            .field("NAND type", &self.get_blk_sz())
            .field("op_scope", &self.get_op_scope())
            .field("ID len", &self.get_id_len())
            .finish()
    }
}

#[allow(unused)]
impl AddrC {
    impl_get_set_field!(get_offset, set_offset, 0..=13);
}
#[allow(unused)]
impl AddrR {
    impl_get_set_field!(get_page, set_page, 0..=24);
    pub fn set_cs(&mut self, chip: usize, param: NandParam) -> &mut Self {
        let addr_cs = chip * (1 << CAP2CS[(NAND_TYPE.get_page_size() >> 8) & 0xf]);
        let blk_sz: usize = param.get_blk_sz().try_into().unwrap();
        self.bits |= (chip * (1 << (16 + blk_sz))) as u32;
        self
    }
}
#[allow(unused)]
impl NandTiming {
    impl_get_set_field!(get_hold_time, set_hold_time, 8..=15);
    impl_get_set_field!(get_wait_cycle, set_wait_cycle, 0..=7);
}

#[allow(unused)]
impl NandParam {
    impl_get_set_field!(get_op_scope, set_op_scope, 16..=29);
    impl_get_set_field!(get_id_len, set_id_len, 12..=14);
    impl_get_set_enum!(get_blk_sz, set_nand_type, 8..=11, NandType);
}

#[allow(unused)]
impl CsRdyMap {
    impl_get_set_field!(get_rdy3_sel, set_rdy3_sel, 28..=31);
    impl_get_set_field!(get_cs3_sel, set_cs3_sel, 24..=27);
    impl_get_set_field!(get_rdy2_sel, set_rdy2_sel, 28..=31);
    impl_get_set_field!(get_cs2_sel, set_cs2_sel, 24..=27);
    impl_get_set_field!(get_rdy1_sel, set_rdy1_sel, 28..=31);
    impl_get_set_field!(get_cs1_sel, set_cs1_sel, 24..=27);
}

#[allow(unused)]
impl NandCmd {
    impl_predicate!(is_dma_req, 31);
    impl_predicate!(is_ecc_dma_req, 30);

    #[inline(always)]
    pub fn is_nand_ce(&self, chip: usize) -> bool {
        debug_assert!(chip <= 3);
        !self.get_bit(chip + 20)
    }

    #[inline(always)]
    pub fn is_nand_rdy(&self, chip: usize) -> bool {
        debug_assert!(chip <= 3);
        self.get_bit(chip + 16)
    }

    impl_get_set!(get_wait_ecc, set_wait_ecc, 14);
    impl_get_set!(get_interrupt_enabled, set_interrupt_enabled, 13);
    impl_get_set!(is_rs_write, set_rs_write, 12);
    impl_get_set!(is_rs_read, set_rs_read, 11);
    impl_get_set!(is_done, set_done, 10);
    impl_get_set!(is_spare, set_spare, 9);
    impl_get_set!(is_main, set_main, 8);
    impl_get_set!(get_read_status, set_read_status, 7);
    impl_get_set!(get_reset, set_reset, 6);
    impl_get_set!(get_read_id, set_read_id, 5);
    impl_get_set!(is_block_erase, set_block_erase, 4);
    impl_get_set!(is_erase_op, set_erase_op, 3);
    impl_get_set!(is_write_op, set_write_op, 2);
    impl_get_set!(is_read_op, set_read_op, 1);
    impl_get_set!(is_cmd_valid, set_cmd_valid, 0);
}

pub struct Nand {
    base: usize,
    chip: usize,
}
impl Debug for NandOpNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NandOpNum")
            .field("bits", &self.bits)
            .finish()
    }
}
impl Debug for NandTiming {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NandTiming")
            .field("hold_time", &self.get_hold_time())
            .field("wait_cyc", &self.get_wait_cycle())
            .finish()
    }
}
impl Debug for CsRdyMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CsRdyMap")
            .field("bits", &self.bits)
            .finish()
    }
}
impl Debug for IdL {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IdL").field("bits", &self.bits).finish()
    }
}
impl Debug for StatusAndIdH {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StatusAndIdH")
            .field("status", &self.get_status())
            .finish()
    }
}
impl Debug for AddrR {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AddrR")
            .field("bits", &self.get_page())
            .finish()
    }
}
impl Debug for AddrC {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AddrC").field("bits", &self.bits).finish()
    }
}
impl Debug for Nand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Nand")
            .field("base", &self.base)
            .field("chip", &self.chip)
            .field("", &self.nand_cmd())
            .field("", &self.addr_c())
            .field("", &self.addr_r())
            .field("", &self.nand_timing())
            .field("", &self.id_l())
            .field("", &self.status_and_id_h())
            .field("", &self.nand_param())
            .field("", &self.nand_op_num())
            .field("", &self.cs_rdy_map())
            .finish()
    }
}
const CAP2CS: [usize; 13] = [16, 17, 19, 19, 19, 19, 20, 21, 14, 15, 16, 17, 18];
#[repr(usize)]
#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
pub enum NandType {
    Sz1GbPg2K = 0,
    Sz2GbPg2K,
    Sz4GbPg2K,
    Sz8GbPg2K,
    Sz16GbPg4K,
    Sz32GbPg8K,
    Sz64GbPg8K,
    Sz128GbPg8K,
    Sz64MbPg512B = 9,
    Sz128MbPg512B,
    Sz256MbPg512B,
    Sz512MbPg512B,
    Sz1GbPg512B,
    Reserved,
}
impl NandType {
    #[inline(always)]
    fn get_ecc_size(&self) -> usize {
        (self.get_page_size() / 512) * 16 - self.get_page_size() / 512
    }
    #[inline(always)]
    fn get_ecc_tot_size(&self) -> usize {
        2 + self.get_page_size() + self.get_ecc_size()
    }
    #[inline(always)]
    fn get_spare_size(&self) -> usize {
        match self.get_page_size() {
            512 => 16,
            2048 => 64,
            4096 => 128,
            8192 => 640,
            _ => 1usize.wrapping_neg(),
        }
    }
    #[inline(always)]
    fn get_page_size(&self) -> usize {
        match self {
            NandType::Sz1GbPg2K => 2 * 1024,
            NandType::Sz2GbPg2K => 2 * 1024,
            NandType::Sz4GbPg2K => 2 * 1024,
            NandType::Sz8GbPg2K => 2 * 1024,
            NandType::Sz16GbPg4K => 4 * 1024,
            NandType::Sz32GbPg8K => 8 * 1024,
            NandType::Sz64GbPg8K => 8 * 1024,
            NandType::Sz128GbPg8K => 8 * 1024,
            NandType::Sz64MbPg512B => 512,
            NandType::Sz128MbPg512B => 512,
            NandType::Sz256MbPg512B => 512,
            NandType::Sz512MbPg512B => 512,
            NandType::Sz1GbPg512B => 512,
            NandType::Reserved => 1usize.wrapping_neg(),
        }
    }
}
#[allow(unused)]
impl Nand {
    impl_get_reg!(nand_cmd, NandCmd);
    impl_get_reg!(addr_c, AddrC);
    impl_get_reg!(addr_r, AddrR);
    impl_get_reg!(nand_timing, NandTiming);
    impl_get_reg!(id_l, IdL);
    impl_get_reg!(status_and_id_h, StatusAndIdH);
    impl_get_reg!(nand_param, NandParam);
    impl_get_reg!(nand_op_num, NandOpNum);
    impl_get_reg!(cs_rdy_map, CsRdyMap);
    impl_get_reg!(dma_acc_addr, DMAAccAddr);
}

impl Nand {
    pub fn new(base: usize, chip: usize) -> Self {
        Self { base, chip }
    }
    #[inline(always)]
    pub fn get_base(&self) -> usize {
        self.base
    }
    fn wait_nand_done(&self, busy: bool) {
        //        let mut tle = 0;
        let base = self.get_base();
        loop {
            let nand_cmd = self.nand_cmd();
            /* debug!(
             *     "[wait_nand_done] {:?}, {:?}",
             *     unsafe { super::dma::DMA_DESC },
             *     self
             * ); */
            if nand_cmd.is_done() {
                self.nand_cmd().set_done(false).write(base);
                break;
            }
            if !busy {
                try_yield();
            }
        }
        NandCmd::empty().write(base);
    }
    #[allow(unused)]
    pub fn hw_init(&mut self) {
        let base = self.get_base();
        let hold_cyc = 0x04;
        let wait_cyc = 0x12;
        NandCmd::empty().write(base);
        AddrC::empty().write(base);
        AddrR::empty().write(base);
        NandTiming::empty()
            .set_hold_time(hold_cyc)
            .set_wait_cycle(wait_cyc)
            .write(base);
        NandParam::empty().set_value(0x08006000).write(base);
        NandOpNum::empty().write(base);
        CsRdyMap::empty().set_value(0x88442200).write(base);
    }
    #[allow(unused)]
    pub fn reset(&mut self) {
        self.setup(
            NandCmd::empty().set_reset(true).write(self.base),
            &mut AddrC::empty(),
            &mut AddrR::empty(),
            &mut NandParam::empty(),
            &mut NandOpNum::empty(),
        );
        self.wait_nand_done(false);
    }
    fn setup(
        &self,
        cmd: &mut NandCmd,
        addr_c: &mut AddrC,
        addr_r: &mut AddrR,
        param: &mut NandParam,
        op_num: &mut NandOpNum,
    ) {
        let base = self.get_base();
        param.write(base);
        op_num.write(base);
        addr_c.write(base);
        addr_r.set_cs(self.chip, *param).write(base);
        NandCmd::empty().write(base);
        cmd.set_cmd_valid(false).write(base);
        cmd.set_cmd_valid(true).write(base);
    }
    pub fn read(
        &self,
        dma: &mut DMAOrder,
        buf: &mut [u8],
        pg: usize,
        en_ecc: bool,
    ) -> Result<(), ()> {
        let base = self.get_base();
        if en_ecc {
            todo!();
            NandTiming::read(base).set_wait_cycle(0x8).write(base);
            let mut param = self.nand_param();
            param
                .set_nand_type(NAND_TYPE)
                .set_op_scope(NAND_TYPE.get_page_size())
                .set_id_len(0);
            self.setup(
                NandCmd::empty()
                    .set_read_op(true)
                    .set_rs_read(true)
                    .set_spare(true),
                AddrC::empty().set_offset(0),
                AddrR::empty().set_page(pg).set_cs(0, param),
                &mut param,
                NandOpNum::empty().set_value(NAND_TYPE.get_spare_size() as u32),
            );
            self.wait_nand_done(false);
        } else {
            NandTiming::read(base).set_wait_cycle(0x12).write(base);
        }
        let mut param = self.nand_param();
        param
            .set_nand_type(NAND_TYPE)
            .set_op_scope(NAND_TYPE.get_page_size());
        self.setup(
            NandCmd::empty()
                .set_read_op(true)
                .set_main(true)
                .set_spare(false),
            AddrC::empty().set_offset(0),
            &mut AddrR::empty().set_page(pg).set_cs(0, param),
            &mut param,
            NandOpNum::empty().set_value(NAND_TYPE.get_page_size() as u32),
        );
        /* debug!("[read] {:?}", self); */
        dma.receive(base + NAND_DMA_BUFFER, buf)?;
        /* debug!("wait_nand_done"); */
        self.wait_nand_done(false);
        Ok(())
    }
    pub fn page_program(&self, dma: &mut DMAOrder, buf: &[u8], pg: usize) -> Result<(), ()> {
        let base = self.get_base();
        /* NandTiming::read(base)
         *     .set_wait_cycle(0x12)
         *     .write(base); */
        let mut param = NandParam::empty();
        param
            .set_nand_type(NAND_TYPE)
            .set_op_scope(NAND_TYPE.get_spare_size() + NAND_TYPE.get_page_size());
        self.setup(
            &mut NandCmd::empty()
                //                .set_rs_write(true)
                .write(base)
                .set_main(true)
                //                .set_spare(true)
                .set_write_op(true),
            &mut AddrC::empty(),
            &mut AddrR::empty().set_page(pg).set_cs(0, param),
            &mut param,
            &mut NandOpNum::empty().set_value(NAND_TYPE.get_page_size() as u32),
        );
        dma.send(base + NAND_DMA_BUFFER, buf)?;
        self.wait_nand_done(false);
        Ok(())
    }
    pub fn erase(&self, pg: usize, cnt: usize) {
        let base = self.get_base();
        NandTiming::read(base).set_wait_cycle(0x12).write(base);
        self.setup(
            NandCmd::empty().set_erase_op(true).set_block_erase(true),
            &mut AddrC::empty(),
            AddrR::empty().set_page(pg),
            &mut NandParam::empty()
                .set_nand_type(NAND_TYPE)
                .set_op_scope(NAND_TYPE.get_page_size() + NAND_TYPE.get_spare_size()),
            &mut NandOpNum::empty().set_value(cnt as u32),
        );
        self.wait_nand_done(false)
    }
}
