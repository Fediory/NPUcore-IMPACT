use core::fmt::Debug;

impl_define_csr!(PGD, " Page Global Directory Base Address (PGD)
This register is a read-only register.
Store global directory base address information corresponding to the bad virtual address in the current context.
");
impl_read_csr!(0x1b, PGD);
impl core::fmt::Debug for PGD {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PGD")
            .field("bits", &format_args!("{:X}", self.bits))
            .finish()
    }
}
impl PGD {
    /// If the highest bit of the bad virtual address(`BadV`) in the current context is 0:
    /// * the return value of reading is equal to the Base field of `CSR.PGDL`;
    /// OTHERWISE,
    /// * the read return value is equal to the Base field of `CSR.PGDH`.
    ///
    /// When `CSR.TLBRERA.IsTLBR`=0,
    /// * the bad virtual address information in the current context is located in `CSR.BADV`;
    /// OTHERWISE,
    /// * the bad virtual address information is located in `CSR.TLBRBADV`.
    pub fn get_base(&self) -> usize {
        self.bits
    }
}

impl_define_csr!(PGDH, "Page Global Directory Base Address for Higher Half Address Space\n\
                        This register is used to configure the base address of the global directory for the lower half address space.\n\
                        It is required that the base address of the global directory must be aligned to a 4KB bound address.\n\
                        This register also contains the information related to the PS and P fields in the TLB table entry when executing the TLB-related instructions.
");
impl_read_csr!(0x1a, PGDH);
impl_write_csr!(0x1a, PGDH);
impl PGDH {
    /// The base address of the global directory in the lower half address space.
    /// By lower half address space, it means that the [VALEN-1] bit of the virtual address is equal to 0.
    pub fn get_base(&self) -> usize {
        self.bits
    }
    /// Set the base *ADDRESS* of the global directory in the higher half address space.
    /// # Warning!
    /// The address MUST be 4K page aligned.
    pub fn set_base(&mut self, val: usize) -> &mut Self {
        // 确保地址是 4KB 边界地址对齐的
        debug_assert_eq!(val & 0xFFF, 0);
        self.bits = val;
        self
    }
}

impl_define_csr!(PGDL, "Page Global Directory Base Address for Lower Half Address Space\n\
                        This register is used to configure the base address of the global directory for the lower half address space.\n\
                        It is required that the base address of the global directory must be aligned to a 4KB bound address.\n\
                        This register also contains the information related to the PS and P fields in the TLB table entry when executing the TLB-related instructions.
");
impl_read_csr!(0x19, PGDL);
impl_write_csr!(0x19, PGDL);

impl PGDL {
    /// The base address of the global directory in the lower half address space.
    /// By lower half address space, it means that the [VALEN-1] bit of the virtual address is equal to 0.
    pub fn get_base(&self) -> usize {
        self.bits
    }
    /// Set the base address of the global directory in the lower half address space.
    /// # Warning!
    /// The address MUST be 4K page aligned.
    pub fn set_base(&mut self, val: usize) -> &mut Self {
        // 确保地址是 4KB 边界地址对齐的
        debug_assert_eq!(val & 0xFFF, 0);
        self.bits = val;
        self
    }
}
impl From<usize> for PGDL {
    fn from(value: usize) -> Self {
        PGDL { bits: value }
    }
}

impl core::fmt::Debug for PGDH {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PGDH")
            .field("bits", &format_args!("{:X}", self.bits))
            .finish()
    }
}
impl core::fmt::Debug for PGDL {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PGDL")
            .field("bits", &format_args!("{:X}", self.bits))
            .finish()
    }
}
