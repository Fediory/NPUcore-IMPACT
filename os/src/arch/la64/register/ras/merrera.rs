impl_define_csr!(MErrEra,"Machine Error Exception Data Save Register\n\
                          This register is used to record the PC of the instruction that triggered the machine error exception.");
impl_read_csr!(0x94, MErrEra);
impl_write_csr!(0x94, MErrEra);

impl MErrEra {
    pub fn get_pc(&self) -> usize {
        self.bits
    }
    pub fn set_pc(&mut self, pc: usize) -> &mut Self {
        self.bits = pc;
        self
    }
}
