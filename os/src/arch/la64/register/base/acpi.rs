use bit_field::BitField;

use crate::arch::board::ACPI_BASE;

const PM1_CNT_ADDR: usize = ACPI_BASE + 0x14;

impl_define_mem_reg!(Pm1Cnt, PM1_CNT_ADDR, "Power Management 1 Control Register ");

impl Pm1Cnt {
    impl_get_set!(
        get_slp_en,
        set_slp_en,
        13,
        "write 1 to set SLP_TYP into sleep，and auto-change to 0 while sleeping"
    );
    impl_get_set!(get_slp_typ, set_slp_typ, 10..=12, "3bit - sleep type");
    pub fn set_s5(&mut self) -> &mut Self {
        self.set_slp_typ(SleepType::S5.into());
        self.set_slp_en(true);
        self
    }
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
#[repr(usize)]
pub enum SleepType {
    /// Full-worked
    S0 = 0b000,
    /// Suspend to RAM(STR)
    S3 = 0b101,
    /// Suspend to Disk(STD)
    S4 = 0b110,
    /// Soft off
    S5 = 0b111,
}
