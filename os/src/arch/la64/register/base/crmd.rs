use core::convert::{TryFrom, TryInto};

use bit_field::BitField;

use crate::arch::la64::register::MemoryAccessType;

// 当前模式信息
impl_define_csr!(
    CrMd,
    "Current Mode Information (CRMD)
The information in this register is used to determine the the processor core’s privilege level,
global interrupt enable bit, watchpoint enable bit, and address translation mode at that time.
"
);
impl_write_csr!(0x0, CrMd);
impl_read_csr!(0x0, CrMd);
impl core::fmt::Debug for CrMd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrMd")
            .field("plv", &self.get_plv())
            .field("ie", &self.is_interrupt_enabled())
            .field("we", &self.is_watchpoint_enabled())
            .field("is_paging_md", unsafe { &self.pg() })
            .field("is_dir_acc", unsafe { &self.da() })
            .field("datf", &self.get_datf())
            .field("datm", &self.get_datm())
            .finish()
    }
}
impl CrMd {
    // 返回整个寄存器的内容
    pub fn bits(&self) -> usize {
        self.bits
    }
    pub fn set_bits(&mut self, bits: usize) -> &mut Self {
        self.bits = bits;
        self
    }
    /// Current privilege level. The legal value range is 0 to 3,
    ///
    /// where 0 is the highest privilege level and 3 is the lowest privilege level.
    ///
    /// When an exception is triggered, the hardware sets this field to 0 to jump to the highest privilege level.
    ///
    /// When the ERTN instruction is executed to return from the exception handler,
    ///
    /// Three potential sources of this value is described as follows:
    /// 1. if CSR.MERRCTL.IsMERR=1, the hardware restores the value of the PPLV field of CSR.MERRCTL here;
    /// 2. otherwise, if CSR.TLBRERA.IsTLBR=1, the hardware restores the value of the PPLV field of CSR.TLBRPRMD to here;
    /// 3. finally, if neither of the two previous values are 1, hardware restores the value of the PPLV field of CSR.PRMD to here.
    pub fn get_plv(&self) -> usize {
        self.bits.get_bits(0..2)
    }
    // Set current privilege level.
    pub fn set_plv(&mut self, mode: usize) -> &mut Self {
        debug_assert!(mode < 4);
        self.bits.set_bits(0..2, mode as usize);
        self
    }
    /// Set the interrupt enabling status to `status`
    pub fn set_ie(&mut self, status: bool) -> &mut Self {
        self.bits.set_bit(2, status);
        self
    }
    /// True if the machine is globally interrupt enabled.
    /// Otherwise false.
    pub fn is_interrupt_enabled(&self) -> bool {
        self.bits.get_bit(2)
    }
    /// Predicate for paging mode
    /// # Return Value
    /// `true` for paging mode on, `false` for direct access mode.
    /// # Panic
    /// *WARNING* the function panics if the `da` and `pg` are equal.
    pub fn is_paging(&self) -> bool {
        unsafe {
            debug_assert_ne!(self.pg(), self.da());
        }
        unsafe { self.pg() }
    }
    /// Set the true
    /// # Arguments
    /// * `on`: `true` to set the paging on and `false` otherwise.
    pub fn set_paging(&mut self, on: bool) -> &mut Self {
        unsafe {
            self.set_pg(on);
            self.set_da(!on);
        }
        self
    }

    /// The memory access type (MAT) for fetch operations in direct address translation mode.
    ///
    /// The hardware sets this field to 0 when a machine error exception is triggered.
    ///
    /// When the execution of the ERTN instruction returns from the exception handler and CSR.MERRCTL.IsMERR=1,
    /// the hardware restores the value of the PDATF field of CSR.MERRCTL to here.
    pub fn get_datf(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(5..=6)).unwrap()
    }
    /// Set memory access type in direct access mode.
    /// # Warning!
    ///
    /// In the case of using software to handle TLB refill, when the software sets PG to 1,
    /// it's a MUST to set the DATF field to 0b01(Coherent Cacheable, aka. CC) at the same time.
    /// See also: `get_datf()`
    pub fn set_datf(&mut self, datf: usize) -> &mut Self {
        self.bits.set_bits(5..=6, datf);
        self
    }
    /// The Memory Access Type(MAT) for load and store operations when in direct address translation mode.
    /// The field is set to 0(Strongly-ordered UnCached (SUC)) in case of a machine error exception.
    /// If `ERTN` instruction returns from the exception handler, and `CSR.MERRCTL.IsMERR`=1,
    /// the hardware restores the value of the `PDATM` field of `CSR.MERRCTL` to here.
    pub fn get_datm(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(7..=8)).unwrap()
    }
    /// Set Memory Access Type (MAT) for load and store operations when in direct address translation mode.
    /// # Warning!
    /// For software TLB refill, when the software sets `PG` to `1`,
    /// it's a MUST to set `DATM` to `0b01`(Coherent Cacheable, aka. CC) at the same time.
    pub fn set_datm(&mut self, datm: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(7..=8, datm.try_into().unwrap());
        self
    }
    /// Instruction and data watchpoints enable bit, which is active high.
    /// The hardware sets the value of this field to 0 when an exception is triggered.
    /// When the ERTN instruction is executed to return from the exception handler.
    /// If `CSR.MERRCTL.IsMERR`=1, the hardware restores the PWE field of CSR.MERRCTL here;
    /// otherwise, if `CSR.TLBRERA.IsTLBR`=1, the hardware restores the PWE field of CSR.TLBRPRMD here;
    /// otherwise, the hardware restores the value of the PWE field of CSR.PRMD here.
    pub fn is_watchpoint_enabled(&self) -> bool {
        // 第9位
        self.bits.get_bit(9)
    }
    /// Instruction and data watchpoints enable bit, which is active high.
    /// The hardware sets the value of this field to 0 when an exception is triggered.
    /// When the ERTN instruction is executed to return from the exception handler.
    /// If `CSR.MERRCTL.IsMERR`=1, the hardware restores the PWE field of CSR.MERRCTL here;
    /// otherwise, if `CSR.TLBRERA.IsTLBR`=1, the hardware restores the PWE field of CSR.TLBRPRMD here;
    /// otherwise, the hardware restores the value of the PWE field of CSR.PRMD here.
    pub fn set_watchpoint_enabled(&mut self, we: bool) -> &mut Self {
        self.bits.set_bit(9, we);
        self
    }

    /// Predict for direct memory access mode.
    pub unsafe fn da(&self) -> bool {
        // 第3位
        self.bits.get_bit(3)
    }
    // 获取PG
    // 第4位
    pub unsafe fn pg(&self) -> bool {
        self.bits.get_bit(4)
    }
    // 设置直接地址翻译使能
    pub unsafe fn set_da(&mut self, da: bool) -> &mut Self {
        self.bits.set_bit(3, da);
        self
    }
    // 设置PG,页翻译使能
    pub unsafe fn set_pg(&mut self, pg: bool) -> &mut Self {
        self.bits.set_bit(4, pg);
        self
    }
}
