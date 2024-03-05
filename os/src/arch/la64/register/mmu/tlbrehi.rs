// 无论 CSR.TLBRERA.IsTLBR 等于何值，执行 TLBRD 指令都只更新 TLBEHI 寄存器
use bit_field::BitField;
use core::fmt::Debug;

use super::VALEN;
impl_define_csr!(TLBREHi,"
When in the TLB refill exception context (`CSR.TLBRERA.IsTLBR`=1),
the `TLBREHI` register stores the information related to the physical page number of the low-order bits of the TLB table entry,
so as to during executing TLB-related instructions, etc.
The format of the `TLBREHI` register and the meaning of each field are the same as the `TLBEHI` register.

However, the `TLBREHI` register is not an exact replica of the `TLBEHI` register in the case of `CSR.TLBRERA.IsTLBR`=1. This is reflected in:

* Regardless of the value of CSR.TLBRERA.IsTLBR equals, the execution of the TLBRD instruction updates only the TLBEHI register.

");
impl_read_csr!(0x8e, TLBREHi);
impl_write_csr!(0x8e, TLBREHi);

impl Debug for TLBREHi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TLBREHi")
            .field("pg_size", &self.get_page_size())
            .field("vppn_head", &format_args!("{:#X}", self.get_vppn() * 2))
            .finish()
    }
}
impl TLBREHi {
    /// Page size specified for `TLB` refill exception.
    /// That is, if `CSR.TLBRERA.IsTLBR`=1,
    /// `TLBWR` or `TLBFILL` instructions write this `PS` field into the TLB entry.
    pub fn get_page_size(&self) -> usize {
        1 << self.bits.get_bits(0..=5)
    }
    /// Set the page size used by `TLBWR` & `TLBFILL` when `CSR.TLBRERA.IsTLBR`=1.
    pub fn set_page_size(&mut self, page_size: usize) -> &mut Self {
        self.bits
            .set_bits(0..=5, page_size.trailing_zeros() as usize);
        self
    }
    /// When CSR.TLBRERA.IsTLBR=1, the value of VPPN used for querying TLB when executing TLBSRCH instruction,
    /// and the value of VPPN field of TLB table entry written when executing TLBWR and TLBFILL instructions come from here.
    /// When a TLB refill exception is triggered, the [VALEN-1:13] bits of the virtual address that triggered the exception are recorded here.
    pub fn get_vppn(&self) -> usize {
        self.bits.get_bits(13..VALEN)
    }
    /// Set the `VPPN` used by `TLBSRCH`, `TLBWR` & `TLBFILL` when `CSR.TLBRERA.IsTLBR`=1.
    pub fn set_vppn(&mut self, vppn: usize) -> &mut Self {
        self.bits.set_bits(13..VALEN, vppn);
        self
    }
}
