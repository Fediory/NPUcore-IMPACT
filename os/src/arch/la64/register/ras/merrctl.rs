use bit_field::BitField;
impl_define_csr!(MErrCtl, "Machine Error Controller\n\
                           Since the timing of machine error exceptions cannot be predicted and controlled by the software,\n\
                           a separate set of CSRs is defined for machine error exceptions to preserve other registers when machine error exceptions are triggered,\n\
                           which is used by the system software to save and restore other sites.\n\
                           This set of independent CSRs except MERRERA and MERRSAVE, the rest are concentrated in MERRCTL register.");
impl_write_csr!(0x90, MErrCtl);
impl_read_csr!(0x90, MErrCtl);

impl MErrCtl {
    pub fn is_merr(&self) -> bool {
        self.bits.get_bit(0)
    }
    pub fn is_repairable(&self) -> bool {
        self.bits.get_bit(1)
    }
    pub fn get_pplv(&self) -> usize {
        self.bits.get_bits(2..=3) as usize
    }
    pub fn set_pplv(&mut self, pplv: usize) -> &mut Self {
        self.bits.set_bits(2..=3, pplv);
        self
    }
    /// The previous global interrupt enable (IE) bit in CSR.CRMD
    pub fn get_pie(&self) -> bool {
        self.bits.get_bit(4)
    }
    pub fn set_ie(&mut self, ie: bool) -> &mut Self {
        self.bits.set_bit(4, ie);
        self
    }
    pub fn pwe(&self) -> bool {
        self.bits.get_bit(6)
    }
    pub fn set_pwe(&mut self, pwe: bool) -> &mut Self {
        self.bits.set_bit(6, pwe);
        self
    }
    pub fn pda(&self) -> bool {
        self.bits.get_bit(7)
    }
    pub fn set_pda(&mut self, pda: bool) -> &mut Self {
        self.bits.set_bit(7, pda);
        self
    }
    pub fn ppg(&self) -> bool {
        self.bits.get_bit(8)
    }
    pub fn set_ppg(&mut self, ppg: bool) -> &mut Self {
        self.bits.set_bit(8, ppg);
        self
    }
    pub fn pdatf(&self) -> usize {
        self.bits.get_bits(9..=10)
    }
    pub fn set_pdatf(&mut self, pdatf: usize) -> &mut Self {
        self.bits.set_bits(9..=10, pdatf);
        self
    }
    pub fn pdatm(&self) -> usize {
        self.bits.get_bits(11..=12)
    }
    pub fn set_pdatm(&mut self, pdatm: usize) -> &mut Self {
        self.bits.set_bits(11..=12, pdatm);
        self
    }
    pub fn cause(&self) -> MachineError {
        let code = self.bits.get_bits(13..=15);
        MachineError::from(code)
    }
}
#[derive(Debug, Clone, Copy)]
pub enum MachineError {
    CacheCheckError,
}

impl From<usize> for MachineError {
    fn from(cause: usize) -> Self {
        match cause {
            0x1 => MachineError::CacheCheckError,
            _ => panic!("Unknown machine error"),
        }
    }
}
