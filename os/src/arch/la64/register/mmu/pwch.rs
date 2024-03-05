use core::fmt::Debug;

use bit_field::BitField;
impl_define_csr!(PWCH, "Page Walk Controller for Higher Half Address Space (PWCH)
This register and the information in the `CSR.PWCL` register together define the page table structure used in the operating system.
This information will be used to instruct software or hardware to perform page table walking.
See Multi-level Page Table Structure Supported by page walking for an illustration of the page table structure and walking process.
");
impl_read_csr!(0x1d, PWCH);
impl_write_csr!(0x1d, PWCH);
impl Debug for PWCH {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PWCH")
            .field("dir3_base", &self.get_dir3_base())
            .field("dir3_width", &self.get_dir3_width())
            .field("dir4_base", &self.get_dir4_base())
            .field("dir4_width", &self.get_dir4_width())
            .finish()
    }
}
impl PWCH {
    /// Get the starting address of the level 3 directory.
    pub fn get_dir3_base(&self) -> usize {
        self.bits.get_bits(0..=5)
    }
    /// Set the starting address of the level 3 directory.
    pub fn set_dir3_base(&mut self, dir2_base: usize) -> &mut Self {
        self.bits.set_bits(0..=5, dir2_base);
        self
    }
    /// Get the number of index bits of the level 3 directory. 0 means there is no such level.
    pub fn get_dir3_width(&self) -> usize {
        self.bits.get_bits(6..=11)
    }
    /// Set the number of index bits of the level 3 directory. 0 means there is no such level.
    pub fn set_dir3_width(&mut self, dir2_width: usize) -> &mut Self {
        self.bits.set_bits(6..=11, dir2_width);
        self
    }
    /// Get the starting address of the level 4 directory.
    pub fn get_dir4_base(&self) -> usize {
        self.bits.get_bits(12..=17)
    }
    /// Set the starting address of the level 4 directory.
    pub fn set_dir4_base(&mut self, dir3_base: usize) -> &mut Self {
        self.bits.set_bits(12..=17, dir3_base);
        self
    }
    /// Get the number of index bits of the level 4 directory. 0 means there is no such level.
    pub fn get_dir4_width(&self) -> usize {
        self.bits.get_bits(18..=23)
    }
    /// Set the number of index bits of the level 4 directory. 0 means there is no such level.
    pub fn set_dir4_width(&mut self, dir3_width: usize) -> &mut Self {
        self.bits.set_bits(18..=23, dir3_width);
        self
    }
}
