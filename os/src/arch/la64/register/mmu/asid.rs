use bit_field::BitField;
impl_define_csr!(
    ASId,
    "Address Space ID\n\
     This register contains the ASID information for access operations and TLB-related instructions."
);
impl_read_csr!(0x18, ASId);
impl_write_csr!(0x18, ASId);

impl ASId {
    pub fn get_asid(&self) -> usize {
        self.bits.get_bits(0..=9)
    }
    pub fn set_asid(&mut self, asid: usize) -> &mut Self {
        self.bits.set_bits(0..=9, asid);
        self
    }
    /// The length of the ASID field. It is directly equal to the value of this field.
    /// # NOTE
    /// This field is current constant.
    pub fn get_asid_width(&self) -> usize {
        self.bits.get_bits(16..=23)
    }
}
