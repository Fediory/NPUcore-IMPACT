use core::fmt::Debug;

// 当触发例外时，如果例外类型不是 TLB 重填例外和机器错误例外，硬件会将此时处理器核的特权等级、
// 全局中断使能和监视点使能位保存至例外前模式信息寄存器中，用于例外返回时恢复处理器核的现场
use bit_field::BitField;

impl_define_csr!(
    PrMd,
    "Pre-exception Mode Information (PRMD)
When an exception is triggered,
if the exception type is not TLB refill exception and machine error exception,
the hardware will save the processor core’s `PLV`,`IE` and `WE` bits at that time,
to `PRMD` to restore the processor core to the context when the exception returns.
"
);
impl_read_csr!(0x1, PrMd);
impl_write_csr!(0x1, PrMd);
impl Debug for PrMd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PrMd")
            .field("pplv", &self.get_pplv())
            .field("pie", &self.get_pie())
            .field("pwe", &self.get_pwe())
            .finish()
    }
}
impl PrMd {
    /// In case of a non-TLB-refill and non-machine-error exception,
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
        self.bits.set_bits(0..2, pplv);
        self
    }
    // Record the `CRMD.IE`(Interrupt Enable bit) before the non-TLB-refill and non-machine-error exception.
    pub fn get_pie(&self) -> bool {
        self.bits.get_bit(2)
    }
    /// Set the value of this field to the `IE` field of `CSR.CRMD` for later return right after restoration of `IE` (Interrupt Enable bit),
    ///  from the exception handler through `ERTN` instruction.
    pub fn set_pie(&mut self, pie: bool) -> &mut Self {
        self.bits.set_bit(2, pie);
        self
    }
    /// In case of a non-TLB-refill and non-machine-error exception,
    /// the hardware records the old value of the `WE`(Watchpoint Enable bit) field in `CSR.CRMD` in this field.
    pub fn get_pwe(&self) -> bool {
        self.bits.get_bit(3)
    }
    /// Set the value of this field to the `WE`(Watchpoint Enable bit) field of `CSR.CRMD` for later return from the exception handler,
    /// through `ERTN` instruction and restoration of `WE`(Watchpoint Enable bit).
    pub fn set_pwe(&mut self, pwe: bool) -> &mut Self {
        self.bits.set_bit(3, pwe);
        self
    }
}
