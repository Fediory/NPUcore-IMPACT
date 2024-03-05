use core::{convert::TryFrom, fmt::Debug};

use bit_field::BitField;

use super::MemoryAccessType;

pub trait DMW: BitField {
    /// true indicates that this window can be used for direct mapping address translation at the PLV0 privilege level.
    fn plv0(&self) -> bool {
        self.get_bit(0)
    }
    /// Set this field true to make this window usable for direct mapping address translation at the PLV0 privilege level.
    fn set_plv0(&mut self, this_plv: bool) -> &mut Self {
        self.set_bit(0, this_plv);
        self
    }
    /// true indicates that this window can be used for direct mapping address translation at the PLV0 privilege level.
    fn plv1(&self) -> bool {
        self.get_bit(1)
    }
    /// Set this field true to make this window usable for direct mapping address translation at the PLV0 privilege level.
    fn set_plv1(&mut self, this_plv: bool) -> &mut Self {
        self.set_bit(1, this_plv);
        self
    }
    /// true indicates that this window can be used for direct mapping address translation at the PLV0 privilege level.
    fn plv2(&self) -> bool {
        self.get_bit(2)
    }
    /// Set this field true to make this window usable for direct mapping address translation at the PLV0 privilege level.
    fn set_plv2(&mut self, this_plv: bool) -> &mut Self {
        self.set_bit(2, this_plv);
        self
    }
    /// true indicates that this window can be used for direct mapping address translation at the PLV0 privilege level.
    fn plv3(&self) -> bool {
        self.get_bit(3)
    }
    /// Set this field true to make this window usable for direct mapping address translation at the PLV0 privilege level.
    fn set_plv3(&mut self, this_plv: bool) -> &mut Self {
        self.set_bit(3, this_plv);
        self
    }
    /// Get the memory access type(MAT) to this window.
    fn get_mat(&self) -> MemoryAccessType;

    /// Set the memory access type(MAT) to this window.
    fn set_mat(&mut self, mat: MemoryAccessType) -> &mut Self;
    /// Get the [63:60] bits of the virtual address of the direct mapping window.
    fn get_vseg(&self) -> usize;
    /// Set the [63:60] bits of the virtual address of the direct mapping window.
    fn set_vesg(&mut self, vseg: usize) -> &mut Self;
}

impl_define_csr!(DMW0,"Direct Mapping Configuration Window 0\n\
                       This group sender is involved in completing the direct mapping address translation mode.\n\
                       See Direct Mapped Address Translation Mode for more information about this address translation mode.");
impl_read_csr!(0x180, DMW0);
impl_write_csr!(0x180, DMW0);
impl Debug for DMW0 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMW0")
            .field("MAT", &self.get_mat())
            .field("vseg", &self.get_vseg())
            .field("plv0", &self.plv0())
            .field("plv1", &self.plv1())
            .field("plv2", &self.plv2())
            .field("plv3", &self.plv3())
            .finish()
    }
}
impl Debug for DMW1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMW1")
            .field("MAT", &self.get_mat())
            .field("vseg", &self.get_vseg())
            .field("plv0", &self.plv0())
            .field("plv1", &self.plv1())
            .field("plv2", &self.plv2())
            .field("plv3", &self.plv3())
            .finish()
    }
}
impl Debug for DMW2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMW2")
            .field("MAT", &self.get_mat())
            .field("vseg", &self.get_vseg())
            .field("plv0", &self.plv0())
            .field("plv1", &self.plv1())
            .field("plv2", &self.plv2())
            .field("plv3", &self.plv3())
            .finish()
    }
}
impl Debug for DMW3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMW3")
            .field("MAT", &self.get_mat())
            .field("vseg", &self.get_vseg())
            .field("plv0", &self.plv0())
            .field("plv1", &self.plv1())
            .field("plv2", &self.plv2())
            .field("plv3", &self.plv3())
            .finish()
    }
}
impl DMW for DMW0 {
    fn get_mat(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(4..=5)).unwrap()
    }
    fn set_mat(&mut self, mat: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(4..=5, mat.into());
        self
    }

    fn get_vseg(&self) -> usize {
        self.get_bits(60..=63).bits
    }

    fn set_vesg(&mut self, vseg: usize) -> &mut Self {
        self.bits.set_bits(60..=63, vseg);
        self
    }
}

impl_define_csr!(DMW1, "Direct Mapping Configuration Window 1\n\
                       This group sender is involved in completing the direct mapping address translation mode.\n\
                       See Direct Mapped Address Translation Mode for more information about this address translation mode.");
impl_read_csr!(0x181, DMW1);
impl_write_csr!(0x181, DMW1);

impl DMW for DMW1 {
    fn get_mat(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(4..=5)).unwrap()
    }
    fn set_mat(&mut self, mat: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(4..=5, mat.into());
        self
    }

    fn get_vseg(&self) -> usize {
        self.get_bits(60..=63).bits
    }

    fn set_vesg(&mut self, vseg: usize) -> &mut Self {
        self.bits.set_bits(60..=63, vseg);
        self
    }
}
impl_define_csr!(DMW2,"Direct Mapping Configuration Window 2\n\
                       This group sender is involved in completing the direct mapping address translation mode.\n\
                       See Direct Mapped Address Translation Mode for more information about this address translation mode.");
impl_read_csr!(0x182, DMW2);
impl_write_csr!(0x182, DMW2);

impl DMW for DMW2 {
    fn get_mat(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(4..=5)).unwrap()
    }
    fn set_mat(&mut self, mat: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(4..=5, mat.into());
        self
    }

    fn get_vseg(&self) -> usize {
        self.get_bits(60..=63).bits
    }

    fn set_vesg(&mut self, vseg: usize) -> &mut Self {
        self.bits.set_bits(60..=63, vseg);
        self
    }
}
impl_define_csr!(DMW3,"Direct Mapping Configuration Window 3\n\
                       This group sender is involved in completing the direct mapping address translation mode.\n\
                       See Direct Mapped Address Translation Mode for more information about this address translation mode.");
impl_read_csr!(0x183, DMW3);
impl_write_csr!(0x183, DMW3);

impl DMW for DMW3 {
    fn get_mat(&self) -> MemoryAccessType {
        MemoryAccessType::try_from(self.bits.get_bits(4..=5)).unwrap()
    }
    fn set_mat(&mut self, mat: MemoryAccessType) -> &mut Self {
        self.bits.set_bits(4..=5, mat.into());
        self
    }

    fn get_vseg(&self) -> usize {
        self.get_bits(60..=63).bits
    }

    fn set_vesg(&mut self, vseg: usize) -> &mut Self {
        self.bits.set_bits(60..=63, vseg);
        self
    }
}
