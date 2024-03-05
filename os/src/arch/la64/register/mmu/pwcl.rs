use bit_field::BitField;
impl_define_csr!(PWCL, "Page Walk Controller for Lower Half Address Space (PWCL)

The information in this register and the CSR.PWCH register together define the page table structure used in the operating system.
This information will be used to instruct software or hardware to perform page table walking.
See Multi-level Page Table Structure Supported by page walking for an illustration of the page table structure and walking process.

In LA32, only `PWCL` is implemented , making it a must for PWCL register to contain all the information needed to describe the page table structure.

Thus the last page table and the lowest two levels of the directory starting at no more than 32 bits, a restriction that still exists in LA64.
");
impl_read_csr!(0x1c, PWCL);
impl_write_csr!(0x1c, PWCL);
impl core::fmt::Debug for PWCL {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PWCL")
            .field("ptbase", &self.get_ptbase())
            .field("ptwidth", &self.get_ptwidth())
            .field("dir1_base", &self.get_dir1_base())
            .field("dir1_width", &self.get_dir1_width())
            .field("dir2_base", &self.get_dir2_base())
            .field("dir2_width", &self.get_dir2_width())
            .field("pte_width", &self.get_pte_width())
            .finish()
    }
}
impl PWCL {
    /// Get the start address of the last page table.
    pub fn get_ptbase(&self) -> usize {
        self.bits.get_bits(0..=4)
    }
    /// Set the start address of the last page table.
    pub fn set_ptbase(&mut self, ptbase: usize) -> &mut Self {
        self.bits.set_bits(0..=4, ptbase);
        self
    }

    /// Get the starting address of the lowest level directory.
    pub fn get_ptwidth(&self) -> usize {
        self.bits.get_bits(5..=9)
    }
    /// Set the starting address of the lowest level directory.
    pub fn set_ptwidth(&mut self, ptwidth: usize) -> &mut Self {
        self.bits.set_bits(5..=9, ptwidth);
        self
    }
    /// Get the starting address of the lowest level directory.
    pub fn get_dir1_base(&self) -> usize {
        self.bits.get_bits(10..=14)
    }
    /// Set the starting address of the lowest level directory.
    pub fn set_dir1_base(&mut self, dir1_base: usize) -> &mut Self {
        self.bits.set_bits(10..=14, dir1_base);
        self
    }
    /// Get the number of index bits of the lowest level directory. 0 means there is no such level.
    pub fn get_dir1_width(&self) -> usize {
        self.bits.get_bits(15..=19)
    }
    /// Set the number of index bits of the lowest level directory. 0 means there is no such level.
    pub fn set_dir1_width(&mut self, dir1_width: usize) -> &mut Self {
        self.bits.set_bits(15..=19, dir1_width);
        self
    }
    /// Get the starting address of the next level directory.
    pub fn get_dir2_base(&self) -> usize {
        self.bits.get_bits(20..=24)
    }
    /// Set the starting address of the next level directory.
    pub fn set_dir2_base(&mut self, dir2_base: usize) -> &mut Self {
        self.bits.set_bits(20..=24, dir2_base);
        self
    }
    /// Get the number of index bits of the next lowest level directory. 0 means there is no such level.
    pub fn get_dir2_width(&self) -> usize {
        self.bits.get_bits(25..=29)
    }
    /// Set the number of index bits of the next lowest level directory. 0 means there is no such level.
    pub fn set_dir2_width(&mut self, dir2_width: usize) -> &mut Self {
        self.bits.set_bits(25..=29, dir2_width);
        self
    }
    /// Get the length of each page table entry in the memory. 0 - 64 bit; 1 - 128 bit; 2 - 192 bit; 3 - 256 bit.
    pub fn get_pte_width(&self) -> usize {
        let val = self.bits.get_bits(30..=31);
        match val {
            0 => 64 / 8,
            1 => 128 / 8,
            2 => 192 / 8,
            3 => 256 / 8,
            _ => panic!("invalid pte_width"),
        }
    }
    // Set the length of each page table entry in the memory. 0 - 64 bit; 1 - 128 bit; 2 - 192 bit; 3 - 256 bit.
    pub fn set_pte_width(&mut self, pte_width: usize) -> &mut Self {
        let val = match pte_width {
            8 => 0,
            16 => 1,
            24 => 2,
            32 => 3,
            _ => panic!("invalid pte_width"),
        };
        self.bits.set_bits(30..=31, val);
        self
    }
}
