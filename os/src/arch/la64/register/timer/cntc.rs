use bit_field::BitField;
impl_define_csr!(CntC, "Counter Compensation, \n\
                        This register can be configured by the software to correct the timer’s readout value.\n\
                        The final readout value will be the original `timer_count_val` + `timer_compensation`.\n\
                        It is important to note that configuring this register does not directly change the timer’s count value.\n\
                        In LA32, this register is 32-bit and its value will be sign extended to 64 bits and then added to the original counter value.");
impl_read_csr!(0x43, CntC);
impl_write_csr!(0x43, CntC);

impl CntC {
    /// Software-configurable counter compensation values.
    pub fn get_compensation(&self) -> usize {
        self.bits
    }
    /// Set the software-configurable counter compensation values.
    pub fn set_compensation(&mut self, compensation: usize) -> &mut Self {
        self.bits = compensation;
        self
    }
}
