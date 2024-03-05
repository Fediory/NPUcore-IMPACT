use core::fmt::Debug;

impl_define_csr!(
    TLBRBadV,
    "TLB Refill Exception Bad Virtual Address (TLBRBADV)

This register is used to record the bad virtual address that triggered the TLB refill exception.
"
);
impl_read_csr!(0x89, TLBRBadV);
impl_write_csr!(0x89, TLBRBadV);
impl Debug for TLBRBadV {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TLBRBadV")
            .field("bits", &format_args!("{:X}", self.bits))
            .finish()
    }
}
impl TLBRBadV {
    /// When the TLB refill exception is triggered, the hardware records the bad virtual address here.
    /// For LA64, in this case, if the privilege level that triggered the exception is in 32-bit address mode,
    /// then the high 32 bits of the recorded virtual address will be set to 0.
    pub fn get_vaddr(&self) -> usize {
        self.bits
    }
    /// When the TLB refill exception is triggered, the hardware records the bad virtual address here.
    /// For LA64, in this case, if the privilege level that triggered the exception is in 32-bit address mode,
    /// then the high 32 bits of the recorded virtual address will be set to 0.
    pub fn set_vaddr(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
}
