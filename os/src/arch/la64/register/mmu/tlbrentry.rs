impl_define_csr!(
    TLBREntry,
    " TLB Refill Exception Entry Base Address (TLBRENTRY)

This register is used to configure the entry base address of the TLB refill exception.
Since the processor core enters direct address translation mode after triggering TLB refill exception,
the entry base address filled here should be a physical address.
"
);
impl_read_csr!(0x88, TLBREntry);
impl_write_csr!(0x88, TLBREntry);

impl TLBREntry {
    /// Get the TLB refill entry.
    pub fn get_addr(&self) -> usize {
        self.bits
    }
    /// Get the TLB refill entry.
    /// # Warning!
    /// The `val` must be page aligned.
    pub fn set_addr(&mut self, val: usize) -> &mut Self {
        // 对齐到4kb
        debug_assert_eq!(val & 0xFFF, 0);
        self.bits = val;
        self
    }
}
