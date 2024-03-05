use core::{convert::TryInto, fmt::Debug};

use bit_field::BitField;

use super::estat::Interrupt;
impl_define_csr!(ECfg,"Exception Configuration (ECFG)
This register is used to control the entry calculation method of exceptions and interrupts and the local enable bit of each interrupt.");
impl_write_csr!(0x4, ECfg);
impl_read_csr!(0x4, ECfg);
impl Debug for ECfg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ECfg")
            .field("line_based_interrupt", &self.get_line_based_interrupt())
            .field("entry_margin", &self.get_entries_margin())
            .finish()
    }
}

bitflags! {
    pub struct LineBasedInterrupt:usize {
        ///Software Interrupt 0
        const SWI0=1<<Interrupt::SWI0 as usize;
        ///Software Interrupt 1
        const SWI1=1<<Interrupt::SWI1 as usize;
        ///Hardware Interrupt 0
        const HWI0=1<<Interrupt::HWI0 as usize;
        ///Hardware Interrupt 1
        const HWI1=1<<Interrupt::HWI1 as usize;
        ///Hardware Interrupt 2
        const HWI2=1<<Interrupt::HWI2 as usize;
        ///Hardware Interrupt 3
        const HWI3=1<<Interrupt::HWI3 as usize;
        ///Hardware Interrupt 4
        const HWI4=1<<Interrupt::HWI4 as usize;
        ///Hardware Interrupt 5
        const HWI5=1<<Interrupt::HWI5 as usize;
        ///Hardware Interrupt 6
        const HWI6=1<<Interrupt::HWI6 as usize;
        ///Hardware Interrupt 7
        const HWI7=1<<Interrupt::HWI7 as usize;
        ///Performance Monitor Counter Overflow Interrupt
        const PMCOV=1<<Interrupt::PMCOV as usize;
        ///Timer Interrupt
        const TIMER=1<<Interrupt::Timer as usize;
        ///Inter-Processor Interrupt
        const IPI=1<<Interrupt::IPI as usize;
    }
}
impl ECfg {
    pub fn get_line_based_interrupt(&self) -> LineBasedInterrupt {
        LineBasedInterrupt {
            bits: self.bits & 0b1_1111_1111_1111,
        }
    }
    #[inline(always)]
    pub fn set_line_based_interrupt_vector(&mut self, lie: LineBasedInterrupt) -> &mut Self {
        // 中断位于0-12位,每一位代表一个局部中断
        self.bits |= lie.bits;
        self
    }
    #[inline(always)]
    pub fn turn_off_line_based_interrupt(&mut self, lie: LineBasedInterrupt) -> &mut Self {
        self.bits &= !lie.bits;
        self
    }
    #[inline(always)]
    pub fn turn_off_all_interrupts(&mut self) -> &mut Self {
        self.bits = 0;
        self
    }
    /// The `VS` field in `ECFG`.
    /// Configure the spacing of exceptions and interrupt entries.
    /// * When `VS`=0, all exceptions and interrupts have the same entry base address.
    /// * When `VS`!=0, the entry base address spacing between each exception and interrupt is `2VS` instructions.
    /// Since the TLB refill exceptions and machine error exceptions have separate entry base addresses,
    /// the entry of both exceptions is not affected by the `VS` field.
    pub fn get_entries_margin(&self) -> usize {
        self.bits.get_bits(16..19)
    }
    /// Configure the spacing of exceptions and interrupt entries. When VS=0, all exceptions and interrupts have the same entry base address. When VS!=0, the entry base address spacing between each exception and interrupt is 2VS instructions. Since the TLB refill exceptions and machine error exceptions have separate entry base addresses, the entry of both exceptions is not affected by the VS field.
    pub fn set_entries_margin(&mut self, value: usize) -> &mut Self {
        self.bits.set_bits(16..19, value);
        self
    }
}
