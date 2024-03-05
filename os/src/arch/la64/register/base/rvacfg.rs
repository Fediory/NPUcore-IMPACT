use core::fmt::Debug;

impl_define_csr!(RVACfg, "Reduced Virtual Address Configuration\n\
                          This register is used to control the length of the address being reduced in the virtual address reduction mode.");
impl_write_csr!(0x1f, RVACfg);
impl_read_csr!(0x1f, RVACfg);
impl Debug for RVACfg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RVACfg")
            .field("rbits", &self.get_rbits())
            .finish()
    }
}
impl RVACfg {
    /// The number of the high order bits of the address to be reduced in the virtual address reduction mode.
    /// It can be configured to a value between 0 and 8.
    /// Specially, 0 means that the virtual address reduction mode is disabled.
    /// The processor behavior with `rbits` over 8 is undefined.
    fn get_rbits(&self) -> usize {
        self.bits
    }
    /// The number of the high order bits of the address to be reduced in the virtual address reduction mode.
    /// It can be configured to a value between 0 and 8.
    /// Specially, 0 means that the virtual address reduction mode is disabled.
    /// # Warning!
    /// The processor behavior with `rbits` over 8 is UNDEFINED.
    fn set_rbits(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }
}
