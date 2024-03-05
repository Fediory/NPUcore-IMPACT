use crate::config::PALEN;
use bit_field::BitField;
impl_define_csr!(MErrEntry,"Machine Error Exception Entry Base Address (MERRENTRY)\n\
                            This register is used to configure the entry base address of the machine error exception.\n\
                            Since the processor core enters the direct address translation mode once the machine error exception is triggered,\n\
                            the entry base address filled here should be the physical address.");
impl_write_csr!(0x93, MErrEntry);
impl_read_csr!(0x93, MErrEntry);

impl MErrEntry {
    pub fn get_addr(&self) -> usize {
        self.bits
    }
    pub fn set_addr(&mut self, addr: usize) -> &mut Self {
        debug_assert_eq!(addr & 0xFFF, 0);
        self.bits = addr;
        self.bits &= 0xFFF;
        self.bits &= (1 << PALEN) - 1;
        self
    }
}
