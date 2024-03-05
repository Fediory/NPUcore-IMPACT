impl_define_csr!(BadI, " Bad Instruction\n\
                        This register is used to record the instruction code of the instruction that triggers the synchronous-related exception.\n\
                        The so-called synchronous-related exceptions are all exceptions except the INTerrupt (INT),\n\
                        the Guest CSR Hardware Change exception (GCHC), and the Machine ERRor exception (MERR).");
impl_write_csr!(0x8, BadI);
impl_read_csr!(0x8, BadI);

impl BadI {
    pub fn get_inst(&self) -> usize {
        self.bits
    }
}
