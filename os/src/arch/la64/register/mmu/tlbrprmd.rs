// 当触发 TLB 重填例外时，硬件会将此时处理器核的特权等级、客户机模式、全局中断使能和监视点使
// 能位保存至该寄存器中，用于例外返回时恢复处理器核的现场

use core::fmt::Debug;

use bit_field::BitField;
impl_define_csr!(
    TLBRPrMd,
    "TLB Refill Exception Pre-exception Mode Information (TLBRPRMD)
When a TLB refill exception is triggered,
the hardware saves the processor core’s PLV, Guest mode, global IE, and WE into this register,
for restoration of the processor core accordingly when the exception returns.
"
);
impl_write_csr!(0x8f, TLBRPrMd);
impl_read_csr!(0x8f, TLBRPrMd);
impl Debug for TLBRPrMd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TLBPrMd")
            .field("pplv", &self.get_pplv())
            .field("pie", &self.get_pie())
            .field("pwe", &self.get_pwe())
            .finish()
    }
}
impl TLBRPrMd {
    /// In case of TLB refill,
    /// the hardware records the old value of the `PLV` field in `CSR.CRMD` in this field.
    /// It later restores the value of this field to the PLV field of CSR.CRMD
    /// after `ERTN` instruction is executed to return from the exception handler.
    pub fn get_pplv(&self) -> usize {
        self.bits.get_bits(0..2)
    }
    /// Set the value of this field to the PLV field of `CSR.CRMD` for later return through
    /// `ERTN` instruction  from the exception handler and restoration of PLV.
    pub fn set_pplv(&mut self, pplv: usize) -> &mut Self {
        debug_assert!(pplv < 4);
        //设置特权级
        // 用于在进入用户程序时设置特权级
        self.bits.set_bits(0..2, pplv as usize);
        self
    }
    // Record the `CRMD.IE`(Interrupt Enable bit) before the TLB-refill exception.
    pub fn get_pie(&self) -> bool {
        self.bits.get_bit(2)
    }
    /// Set the value of this field to the `IE` field of `CSR.CRMD` for later return right after restoration of `IE` (Interrupt Enable bit),
    ///  from the exception handler through `ERTN` instruction.
    pub fn set_pie(&mut self, pie: bool) -> &mut Self {
        self.bits.set_bit(2, pie);
        self
    }
    /// In case of a TLB refill exception,
    /// the hardware records the old value of the `WE`(Watchpoint Enable bit) field in `CSR.CRMD` in this field.
    pub fn get_pwe(&self) -> bool {
        self.bits.get_bit(4)
    }
    /// Set the value of this field to the `WE`(Watchpoint Enable bit) field of `CSR.CRMD` for later return from the exception handler,
    /// through `ERTN` instruction and restoration of `WE`(Watchpoint Enable bit).
    pub fn set_pwe(&mut self, pwe: bool) -> &mut Self {
        self.bits.set_bit(4, pwe);
        self
    }
}
