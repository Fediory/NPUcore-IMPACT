use bit_field::BitField;
impl_define_csr!(TLBIdx, "TLB Index (TLBIDX)

This register contains information such as the index associated with the TLB-related instruction.
The length of the Index field in the table depends on implementation,
although LoongArch allows for an Index length of no more than 16 bits.

This register also contains the information related to the PS and P fields in the TLB table entry when executing TLB-related instructions.
");
impl_read_csr!(0x10, TLBIdx);
impl_write_csr!(0x10, TLBIdx);

impl TLBIdx {
    /// When executing the TLBRD and TLBWR instructions, the index of the access TLB table entry comes from here.
    ///
    /// When executing the TLBSRCH instruction, if it hits, the index of the hit entry is recorded here.
    ///
    /// For the correspondence between index values and TLB table entries, refer to the relevant section in TLB Maintenance Instructions.
    pub fn get_index(&self) -> usize {
        self.bits.get_bits(0..16)
    }
    /// Set the index of TLB table entry for TLBRD and TLBWR instructions to access.
    pub fn set_index(&mut self, index: usize) -> &mut Self {
        self.bits.set_bits(0..16, index);
        self
    }
    /// When executing the TLBRD instruction, the value read from the PS field of the TLB table entry is recorded here.

    /// When executing the TLBWR and TLBFILL instructions with `CSR.TLBRERA.IsTLBR=0`,
    /// the value written to the PS field of the TLB table entry comes from here.
    pub fn get_ps(&self) -> usize {
        self.bits.get_bits(24..=29)
    }
    /// Set the page size for TLBRD, TLBFILL and TLBWR (latter two with `CSR.TLBRERA.IsTLBR=0`) to use manually.
    pub fn set_ps(&mut self, ps: usize) -> &mut Self {
        self.bits.set_bits(24..=29, ps);
        self
    }
    #[doc = "1 means the TLB table entry is empty (invalid TLB table entry),
and 0 means the TLB table entry is non-empty (valid TLB table entry)

* When executing the TLBSRCH instruction, this bit is recorded as 0 if there is a hit entry, otherwise it is recorded as 1.

* When executing the TLBRD instruction, the E bit read from the TLB table entry is inverted and recorded here.

* When executing the TLBWR instruction, then
  * If `CSR.TLBRFPC.IsTLBR`=0, the value written to the E bit of the TLB entry is written after it is inverted.
  * else, if `CSR.TLBRERA.IsTLBR`=1, then the E bit of the TLB entry being written is always set to 1, regardless of the value of that bit."]
    pub fn is_non_existent(&self) -> bool {
        self.bits.get_bit(31)
    }
    #[doc = "1 means the TLB table entry is empty (invalid TLB table entry),
and 0 means the TLB table entry is non-empty (valid TLB table entry)

* When executing the TLBSRCH instruction, this bit is recorded as 0 if there is a hit entry, otherwise it is recorded as 1.

* When executing the TLBRD instruction, the E bit read from the TLB table entry is inverted and recorded here.

* When executing the TLBWR instruction, then
  * If `CSR.TLBRFPC.IsTLBR`=0, the value written to the E bit of the TLB entry is written after it is inverted.
  * else, if `CSR.TLBRERA.IsTLBR`=1, then the E bit of the TLB entry being written is always set to 1, regardless of the value of that bit."]
    pub fn set_ne(&mut self, ne: bool) -> &mut Self {
        self.bits.set_bit(31, ne);
        self
    }
}
