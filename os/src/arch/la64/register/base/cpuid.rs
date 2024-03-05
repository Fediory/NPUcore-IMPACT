impl_define_csr!(
    CPUId,
    "This register contains the processor core number information."
);
impl_read_csr!(0x20, CPUId);

impl CPUId {
    pub fn get_core_id(&self) -> usize {
        self.bits
    }
}
