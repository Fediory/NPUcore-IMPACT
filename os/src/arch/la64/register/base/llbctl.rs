use bit_field::BitField;
impl_define_csr!(
    LLBCtl,
    "This register is used for the access control operations performed on the `LLBit`"
);
impl_read_csr!(0x60, LLBCtl);
impl_write_csr!(0x60, LLBCtl);

impl LLBCtl {
    #[doc = "A read-only bit. Reading this bit will return the value of the current LLBit."]
    pub fn ro_llbit(&self) -> bool {
        self.bits.get_bit(0)
    }
    #[doc = "A software writing 1 to this bit will clear the LLBit to 0.
A software writing 0 to this bit will be *ignored* by hardware."]
    pub fn set_wr_clr_llbit(&mut self, clear: bool) -> &mut Self {
        self.bits.set_bit(1, clear);
        self
    }
    /// Keep LLBit unclear once when `ERTN`, clearing `KLO` bit instead.
    pub fn is_klo(&self) -> bool {
        self.bits.get_bit(2)
    }

    /// Set KLB to 1 to cancel hardware clear of `LLBit` from `ERTN` for only one time.
    pub fn set_klo(&mut self, ctr: bool) -> &mut Self {
        self.bits.set_bit(2, ctr);
        self
    }
}
