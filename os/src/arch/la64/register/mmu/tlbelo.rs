pub trait TLBEL: BitField {
    // 页表项的有效位（V）
    fn valid(&self) -> bool {
        self.get_bit(0)
    }

    fn set_valid(&mut self, valid: bool) -> &mut Self {
        self.set_bit(0, valid);
        self
    }

    fn dirty(&self) -> bool {
        self.get_bit(1)
    }

    fn set_dirty(&mut self, dirty: bool) -> &mut Self {
        self.set_bit(1, dirty);
        self
    }
    fn global(&self) -> bool {
        self.get_bit(6)
    }

    fn set_global(&mut self, global_flag: bool) -> &mut Self {
        self.set_bit(6, global_flag);
        self
    }
    fn not_readable(&self) -> bool {
        self.get_bit(61)
    }

    fn set_not_readable(&mut self, not_readable: bool) -> &mut Self {
        self.set_bit(61, not_readable);
        self
    }

    fn not_executable(&self) -> bool {
        self.get_bit(62)
    }

    fn set_not_executable(&mut self, not_executable: bool) -> &mut Self {
        self.set_bit(62, not_executable);
        self
    }

    fn rplv(&self) -> bool {
        self.get_bit(63)
    }

    fn set_rplv(&mut self, rplv: bool) -> &mut Self {
        self.set_bit(63, rplv);
        self
    }

    fn plv(&self) -> usize;
    fn set_plv(&mut self, plv: usize) -> &mut Self;
    fn get_mat(&self) -> MemoryAccessType;
    fn set_mat(&mut self, mem_access_type: MemoryAccessType) -> &mut Self;
    fn get_ppn(&self) -> PhysPageNum;
    fn set_ppn(&mut self, ppn: PhysPageNum) -> &mut Self;
}

use crate::{arch::la64::laflex::LAFlexPageTableEntry, config::PALEN, mm::PhysPageNum};
use bit_field::BitField;
use core::{
    convert::TryFrom,
    fmt::{self, Display},
};

use super::MemoryAccessType;

impl_define_csr!(TLBELO0, "TLB Entry Low-order Bits 
n
TLBELO0 and TLBELO1 registers contain the information related to the physical page number of the low-order bits of the TLB table entry during executing TLB-related instructions.
Since TLB adopts a dual-page structure,
the low-order bits of TLB table entry corresponds to the odd and even physical page table entries,
where the even page information is in TLBELO0 and the odd page information is in TLBELO1.
TLBELO0 and TLBELO1 registers have exactly the same format definition

When CSR.TLBRERA.IsTLBR=0, and when executing the TLBWR and TLBFILL instructions,
and the written values of the G, PFN, V, PLV, MAT, D, NR, NX, RPLV fields of the TLB table entry come from TLBELOO and TLBELO1 fields, respectively.

When executing the TLBRD instruction,
the above information read from the TLB table entry is written to the corresponding fields in the TLBELO0 and TLBELO1 registers one by one.
");
impl_read_csr!(0x12, TLBELO0);
impl_write_csr!(0x12, TLBELO0);

impl fmt::Display for TLBELO0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TlbElo0: RPLV:{},NX:{},NR:{},PPN:{:?},G:{},MAT:{},PLV:{},D:{},V:{}",
            self.rplv(),
            self.not_executable(),
            self.not_readable(),
            self.get_ppn(),
            self.global(),
            self.get_mat(),
            self.plv(),
            self.dirty(),
            self.valid()
        )
    }
}

impl TLBEL for TLBELO0 {
    fn plv(&self) -> usize {
        self.bits.get_bits(2..=3)
    }

    fn set_plv(&mut self, plv: usize) -> &mut Self {
        self.bits.set_bits(2..=3, plv);
        self
    }

    fn get_mat(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(4..=5)).unwrap()
    }

    fn set_mat(&mut self, mem_access_type: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(4..=5, mem_access_type.into());
        self
    }

    fn get_ppn(&self) -> PhysPageNum {
        let u = self.bits.get_bits(12..PALEN);
        PhysPageNum(u)
    }

    fn set_ppn(&mut self, ppn: PhysPageNum) -> &mut Self {
        self.bits.set_bits(12..PALEN, ppn.0);
        self
    }
}

impl_define_csr!(TLBELO1, "TLB Entry Low-order Bits 

TLBELO0 and TLBELO1 registers contain the information related to the physical page number of the low-order bits of the TLB table entry during executing TLB-related instructions.
Since TLB adopts a dual-page structure,
the low-order bits of TLB table entry corresponds to the odd and even physical page table entries,
where the even page information is in TLBELO0 and the odd page information is in TLBELO1.
TLBELO0 and TLBELO1 registers have exactly the same format definition

When CSR.TLBRERA.IsTLBR=0, and when executing the TLBWR and TLBFILL instructions,
and the written values of the G, PFN, V, PLV, MAT, D, NR, NX, RPLV fields of the TLB table entry come from TLBELOO and TLBELO1 fields, respectively.

When executing the TLBRD instruction,
the above information read from the TLB table entry is written to the corresponding fields in the TLBELO0 and TLBELO1 registers one by one.
");
impl_read_csr!(0x13, TLBELO1);
impl_write_csr!(0x13, TLBELO1);

impl Display for TLBELO1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TlbElo1: RPLV:{},NX:{},NR:{},PPN:{:?},G:{},MAT:{},PLV:{},D:{},V:{}",
            self.rplv(),
            self.not_executable(),
            self.not_readable(),
            self.get_ppn(),
            self.global(),
            self.get_mat(),
            self.plv(),
            self.dirty(),
            self.valid()
        )
    }
}
impl TLBEL for TLBELO1 {
    fn plv(&self) -> usize {
        self.bits.get_bits(2..=3)
    }

    fn set_plv(&mut self, plv: usize) -> &mut Self {
        self.bits.set_bits(2..=3, plv);
        self
    }

    fn get_mat(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(4..=5)).unwrap()
    }

    fn set_mat(&mut self, mem_access_type: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(4..=5, mem_access_type.into());
        self
    }
    fn get_ppn(&self) -> PhysPageNum {
        PhysPageNum(self.bits.get_bits(12..PALEN))
    }
    fn set_ppn(&mut self, ppn: PhysPageNum) -> &mut Self {
        self.bits.set_bits(12..PALEN, ppn.0);
        self
    }
}
