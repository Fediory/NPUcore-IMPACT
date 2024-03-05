impl_define_csr!(TLBRSave, " TLB Refill Exception Data Save Register (TLBRSAVE)

This register is used to store data temporarily for the system software.
Each dava save register can hold the data of one general-purpose register.

The reason for the additional SAVE register for TLB refill exception processing is:
To address the case where a TLB refill exception is triggered during the processing of exceptions except the TLB refill exception.
");
impl_write_csr!(0x8b, TLBRSave);
impl_read_csr!(0x8b, TLBRSave);

impl TLBRSave {
    pub fn get_data(&self) -> usize {
        self.bits
    }
    pub fn set_data(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
}
