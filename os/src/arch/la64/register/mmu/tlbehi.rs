use crate::{config::VALEN, mm::VirtPageNum};
use bit_field::BitField;
impl_define_csr!(TLBEHi, "TLB Entry High-order Bits (TLBEHI)

This register contains the information related to the VPN of the high-order bits of the TLB table entry for TLB-related instructions.

Since the length of the `VPPN` field contained in the high-order bits of the TLB table entry is depends on the range of valid virtual addresses supported by the implementation,
the definition of the relevant register field is expressed separately.
");
impl_read_csr!(0x11, TLBEHi);
impl_write_csr!(0x11, TLBEHi);

impl TLBEHi {
    #[doc = "
* When executing the `TLBRD` instruction, the value of the `VPPN` field read from the `TLB` table entry is recorded here. 

* When `CSR.TLBRERA.IsTLBR`=0, the VPPN value used to query `TLB` when executing `TLBSRCH` instruction and the value of VPPN field written to `TLB` table entry when executing `TLBWR` and `TLBFILL` instructions come from here.

* When the page invalid exception for load operation, page invalid exception for store operation, page invalid exception for fetch operation, page modification exception, page non-readable exception, page non-executable exception, and page privilege level ilegal exception are triggered, the [31:13] bits of the virual address that triggered the exception are recorded here.
"]
    pub fn get_vppn(&self) -> VirtPageNum {
        VirtPageNum(self.bits.get_bits(13..VALEN))
    }
    #[doc = "* When executing the `TLBRD` instruction, the value of the `VPPN` field read from the `TLB` table entry is recorded here.

* When `CSR.TLBRERA.IsTLBR`=0, the VPPN value used to query `TLB` when executing `TLBSRCH` instruction and the value of VPPN field written to `TLB` table entry when executing `TLBWR` and `TLBFILL` instructions come from here.

* When the page invalid exception for load operation, page invalid exception for store operation, page invalid exception for fetch operation, page modification exception, page non-readable exception, page non-executable exception, and page privilege level ilegal exception are triggered, the [31:13] bits of the virual address that triggered the exception are recorded here.
"]
    pub fn set_vppn(&mut self, vpn: VirtPageNum) -> &mut Self {
        self.bits.set_bits(13..VALEN, vpn.0 >> 1);
        self
    }
}
