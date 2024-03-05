use bit_field::BitField;
impl_define_csr!(
    STLBPS,
    "STLB Page Size (STLBPS)

This register is used to configure the size of the page in the STLB.
"
);
impl_read_csr!(0x1e, STLBPS);
impl_write_csr!(0x1e, STLBPS);

impl STLBPS {
    /// Get the `log(RealPageSize)/log(2)`
    /// For example, if the page size is 16KB, then `PS`=`0xE`.
    pub fn get_ps(&self) -> usize {
        self.bits.get_bits(0..=5)
    }
    /// Set the `log(RealPageSize)/log(2)`
    /// For example, if the page size is 16KB, then `PS`=`0xE`.
    pub fn set_ps(&mut self, page_size: usize) -> &mut Self {
        self.bits.set_bits(0..=5, page_size);
        self
    }
}
