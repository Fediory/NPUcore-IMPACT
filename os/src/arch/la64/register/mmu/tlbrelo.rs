use core::convert::{TryFrom, TryInto};

use super::{tlbelo::TLBEL, MemoryAccessType};
use crate::arch::la64::laflex::LAFlexPageTableEntry;
use crate::config::PALEN;
use crate::mm::address::PhysPageNum;
use bit_field::BitField;
impl_define_csr!(TLBRELo0,"TLB Refill Exception Entry Low-order Bits (TLBRELO0, TLBRELO1)

The TLBRELO registers store the low-order bits of PPN-related information in the TLB table entry,
during executing TLB-related instructions
(when the TLB refill exception context CSR.TLBRERA.IsTLBR=1).

The format of TLBRELO registers and the meaning of each field are the same as TLBELO registers.

However, the TLBRELO registers are not an exact copy of the TLBELO registers,
in the case of CSR.TLBRERA.IsTLBR=1. This is reflected in two points:

* Regardless of the value of CSR.TLBRERA.IsTLBR, the TLBRD instruction updates only the TLBELO0/TLBELO1 registers.

* Regardless of the value of CSR.TLBRERA.IsTLBR, the LDPTE instruction updates only the TLBRELO0/TLBRELO1 registers.
");
impl_read_csr!(0x8c, TLBRELo0);
impl_write_csr!(0x8c, TLBRELo0);

impl core::fmt::Debug for TLBRELo0 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TLBRELo0")
            .field("bits", &self.bits)
            .field("valid", &self.valid())
            .field("MAT", &self.get_mat())
            .field("NR", &self.not_readable())
            .field("NX", &self.not_executable())
            .field("ppn", &self.get_ppn())
            .field("dirty", &self.dirty())
            .field("rplv", &self.rplv())
            .field("global", &self.global())
            .field("plv", &self.plv())
            .finish()
    }
}
impl core::fmt::Debug for TLBRELo1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TLBRELo1")
            .field("bits", &self.bits)
            .field("valid", &self.valid())
            .field("MAT", &self.get_mat())
            .field("NR", &self.not_readable())
            .field("NX", &self.not_executable())
            .field("ppn", &self.get_ppn())
            .field("dirty", &self.dirty())
            .field("rplv", &self.rplv())
            .field("global", &self.global())
            .field("plv", &self.plv())
            .finish()
    }
}
impl TLBEL for TLBRELo0 {
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
        self.bits
            .set_bits(4..=5, TryInto::try_into(mem_access_type).unwrap());
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

impl_define_csr!(TLBRELo1,"TLB Refill Exception Entry Low-order Bits (TLBRELO0, TLBRELO1)

The TLBRELO registers store the low-order bits of PPN-related information in the TLB table entry,
during executing TLB-related instructions
(when the TLB refill exception context CSR.TLBRERA.IsTLBR=1).

The format of TLBRELO registers and the meaning of each field are the same as TLBELO registers.

However, the TLBRELO registers are not an exact copy of the TLBELO registers,
in the case of CSR.TLBRERA.IsTLBR=1. This is reflected in two points:

* Regardless of the value of CSR.TLBRERA.IsTLBR, the TLBRD instruction updates only the TLBELO0/TLBELO1 registers.

* Regardless of the value of CSR.TLBRERA.IsTLBR, the LDPTE instruction updates only the TLBRELO0/TLBRELO1 registers.
");
impl_read_csr!(0x8d, TLBRELo1);
impl_write_csr!(0x8d, TLBRELo1);

impl TLBEL for TLBRELo1 {
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
        self.bits
            .set_bits(4..=5, mem_access_type.try_into().unwrap());
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
