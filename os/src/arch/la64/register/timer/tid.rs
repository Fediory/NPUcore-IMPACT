impl_define_csr!(
    TId,
    "Timer Id,\n\
     Each timer in the processor has a unique identifiable number, \n\
     which is configured by the software in this register. \n\
     Each timer also uniquely corresponds to a timer, \n\
     and when the software reads the timer value using the RDTIME instruction, \n\
     the timer ID number that is returned along with it is the corresponding timer number."
);
impl_read_csr!(0x40, TId);
impl_write_csr!(0x40, TId);

impl TId {
    /// Timer number.
    /// It can be configured via software. During a processor core reset, the hardware can reset it to the same value as the CoreID in CSR.CPUID.
    pub fn tid(&self) -> u32 {
        self.bits as u32
    }
}
