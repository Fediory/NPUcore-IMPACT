use bit_field::BitField;
use core::ops::Mul;

impl_define_csr!(Misc,"Miscellaneous Controller (MISC)

This register contains a number of control bits for the operating behavior of the processor core at different privilege levels, including whether to enable 32-bit address mode, whether to allow partially privileged instructions at non-privileged levels, whether to enable address non-alignment check, and whether to enable page table write protection check.
");
impl core::fmt::Debug for Misc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Misc")
            .field(
                "32-bit addr plv(1,2,3):",
                &format_args!(
                    "{},{},{}",
                    self.is_va32l1(),
                    self.is_va32l2(),
                    self.is_va32l3()
                ),
            )
            .field(
                "rdtime allowed for plv(1,2,3):",
                &format_args!(
                    "{},{},{}",
                    self.is_drdtl1(),
                    self.is_drdtl2(),
                    self.is_drdtl3()
                ),
            )
            .field(
                "Disable dirty bit check for plv(0,1,2):",
                &format_args!(
                    "{},{},{}",
                    self.is_dwpl0(),
                    self.is_dwpl1(),
                    self.is_dwpl2(),
                ),
            )
            .field(
                "Misalignment check for plv(0,1,2,4):",
                &format_args!(
                    "{},{},{},{}",
                    self.is_alcl0(),
                    self.is_alcl1(),
                    self.is_alcl2(),
                    self.is_alcl3(),
                ),
            )
            .finish()
    }
}
impl Misc {
    /// Whether to enable 32-bit address mode at the PLV1 privilege level.
    pub fn is_va32l1(&self) -> bool {
        self.bits.get_bit(1)
    }
    /// Set to 1 to enable 32-bit address mode at the PLV1 privilege level.
    pub fn set_va32l1(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(1, value);
        self
    }
    /// Whether to enable 32-bit address mode at the PLV2 privilege level.
    pub fn is_va32l2(&self) -> bool {
        self.bits.get_bit(2)
    }
    /// Set to 1 to enable 32-bit address mode at the PLV2 privilege level.
    pub fn set_va32l2(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(2, value);
        self
    }
    /// Whether to enable 32-bit address mode at the PLV3 privilege level.
    pub fn is_va32l3(&self) -> bool {
        self.bits.get_bit(3)
    }
    /// Set to 1 to enable 32-bit address mode at the PLV3 privilege level.
    pub fn set_va32l3(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(3, value);
        self
    }
    /// Whether to disable RDTIME-like instructions at the PLV1 privilege level.
    pub fn is_drdtl1(&self) -> bool {
        self.bits.get_bit(5)
    }
    /// Set this bit to 1, to *DISABLE* execution of an RDTIME-like instruction,
    /// triggering an instruction privilege level error exception (IPE) at the PLV1 privilege level instead.
    pub fn set_drdtl1(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(5, value);
        self
    }
    /// Whether to disable RDTIME-like instructions at the PLV2 privilege level.
    pub fn is_drdtl2(&self) -> bool {
        self.bits.get_bit(6)
    }
    /// Set this bit to 1, to *DISABLE* execution of an RDTIME-like instruction,
    /// triggering an instruction privilege level error exception (IPE) at the PLV2 privilege level instead.
    pub fn set_drdtl2(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(6, value);
        self
    }
    /// Whether to disable RDTIME-like instructions at the PLV3 privilege level.
    pub fn is_drdtl3(&self) -> bool {
        self.bits.get_bit(7)
    }
    /// Set this bit to 1, to *DISABLE* execution of an RDTIME-like instruction,
    /// triggering an instruction privilege level error exception (IPE) at the PLV1 privilege level instead.
    pub fn set_drdtl3(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(7, value);
        self
    }
    /// Whether to allow software reads of the performance counter at the PLV1 privilege level.
    pub fn is_rpcntl1(&self) -> bool {
        self.bits.get_bit(9)
    }
    /// Set this bit to 1, to allow CSRRD access to any of the implemented performance counters in PLV1,
    /// instead of triggering an instruction privilege level error exception (IPE), in PLV1
    pub fn set_rpcntl1(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(9, value);
        self
    }
    /// Whether to allow software reads of the performance counter at the PLV2 privilege level.
    pub fn is_rpcntl2(&self) -> bool {
        self.bits.get_bit(10)
    }
    /// Set this bit to 1, to allow CSRRD access to any of the implemented performance counters in PLV2,
    /// instead of triggering an instruction privilege level error exception (IPE), in PLV2.
    pub fn set_rpcntl2(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(10, value);
        self
    }
    /// Whether to allow software reads of the performance counter at the PLV3 privilege level.
    pub fn is_rpcntl3(&self) -> bool {
        self.bits.get_bit(11)
    }
    /// Set this bit to 1, to allow CSRRD access to any of the implemented performance counters in PLV3,
    /// instead of triggering an instruction privilege level error exception (IPE), in PLV3.
    pub fn set_rpctl3(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(11, value);
        self
    }

    /// Whether to perform a non-alignment check for non-vector load/store instructions that are allowed to be non-aligned at PLV0 privilege level.
    pub fn is_alcl0(&self) -> bool {
        self.bits.get_bit(12)
    }
    /// Set this bit to 1 to indicate that the misalignment check is performed,
    /// and an address alignment error exception is triggered if illegal, in PLV0.
    ///
    /// This bit is read/write only if the  non-aligned addresses for non-vector load/store instructions is supported.
    ///
    /// Otherwise, the bit is a read-only constant 1.
    pub fn set_alcl0(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(12, value);
        self
    }
    /// Whether to perform a non-alignment check for non-vector load/store instructions that are allowed to be non-aligned at PLV1 privilege level.
    pub fn is_alcl1(&self) -> bool {
        self.bits.get_bit(13)
    }

    /// Set this bit to 1 to indicate that the misalignment check is performed,
    /// and an address alignment error exception is triggered if illegal, in PLV1.
    ///
    /// This bit is read/write only if the  non-aligned addresses for non-vector load/store instructions is supported.
    ///
    /// Otherwise, the bit is a read-only constant 1.
    pub fn set_alcl1(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(13, value);
        self
    }
    /// Whether to perform a non-alignment check for non-vector load/store instructions that are allowed to be non-aligned at PLV2 privilege level.
    pub fn is_alcl2(&self) -> bool {
        self.bits.get_bit(14)
    }

    /// Set this bit to 1 to indicate that the misalignment check is performed,
    /// and an address alignment error exception is triggered if illegal, in PLV2.
    ///
    /// This bit is read/write only if the  non-aligned addresses for non-vector load/store instructions is supported.
    ///
    /// Otherwise, the bit is a read-only constant 1.
    pub fn set_alcl2(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(14, value);
        self
    }
    /// Whether to perform a non-alignment check for non-vector load/store instructions that are allowed to be non-aligned at PLV3 privilege level.
    pub fn is_alcl3(&self) -> bool {
        self.bits.get_bit(15)
    }
    /// Set this bit to 1 to indicate that the misalignment check is performed,
    /// and an address alignment error exception is triggered if illegal in PLV3.
    ///
    /// This bit is read/write only if the  non-aligned addresses for non-vector load/store instructions is supported.
    ///
    /// Otherwise, the bit is a read-only constant 1.
    pub fn set_alcl3(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(15, value);
        self
    }
    /// Whether to disable the check of the page table entry write protection during TLB virtual and real address translation at the PLV0 privilege level.
    pub fn is_dwpl0(&self) -> bool {
        self.bits.get_bit(16)
    }
    /// Set this bit to 1, to disable a page modification exception for store instructions on accessing a page table entry with D=0.
    pub fn set_dwpl0(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(16, value);
        self
    }
    /// Whether to disable the check of the page table entry write protection during TLB virtual and real address translation at the PLV1 privilege level.
    pub fn is_dwpl1(&self) -> bool {
        self.bits.get_bit(17)
    }
    /// Set this bit to 1, to disable a page modification exception for store instructions on accessing a page table entry with D=0 in PLV1.
    pub fn set_dwpl1(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(17, value);
        self
    }
    /// Whether to disable the check of the page table entry write protection during TLB virtual and real address translation at the PLV2 privilege level.
    pub fn is_dwpl2(&self) -> bool {
        self.bits.get_bit(18)
    }
    /// Set this bit to 1, to disable a page modification exception for store instructions on accessing a page table entry with D=0 in PLV2.
    pub fn set_dwpl2(&mut self, value: bool) -> &mut Self {
        self.bits.set_bit(18, value);
        self
    }
}

impl_write_csr!(0x3, Misc);
impl_read_csr!(0x3, Misc);
