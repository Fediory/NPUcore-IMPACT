use bit_field::BitField;
impl_define_csr!(EUEn,"Extended Component Unit Enable (EUEN)

In addition to the base integer instruction set and the privileged instruction set,
the base floating-point instruction set,
the binary translation extension instruction set,
the 128-bit vector extension instruction set,
and the 256-bit vector extension instruction set each have software-configurable enable bits.

When these enable controls are disabled, execution of the corresponding instruction will trigger the corresponding instruction unavailable exception.

Software uses this mechanism to determine the scope when saving the context.

Hardware implementations can also use the control bits here to implement circuit power control.
");

impl_write_csr!(0x2, EUEn);
impl_read_csr!(0x2, EUEn);
impl EUEn {
    #[doc = "The base floating-point instruction enable bit.
If disabled, execution of the base floating-point instruction will trigger a floating-point instruction disable exception (FPD)."]
    pub fn is_float_point_enabled(&self) -> bool {
        self.bits.get_bit(0)
    }
    #[doc = "The base floating-point instruction enable bit.
If disabled, execution of the base floating-point instruction will trigger a floating-point instruction disable exception (FPD)."]
    pub fn set_float_point_stat(&mut self, fpe: bool) -> &mut Self {
        self.bits.set_bit(0, fpe);
        self
    }

    #[doc = "The base SIMD extension instruction enable bit.
If disabled, execution of the SIMD instruction will trigger an SIMD extension instruction disable exception (SXD)."]
    pub fn is_simd_extension_enabled(&self) -> bool {
        self.bits.get_bit(1)
    }
    #[doc = "The base SIMD extension instruction enable bit.
If disabled, execution of the SIMD instruction will trigger an SIMD extension instruction disable exception (SXD)."]
    pub fn set_simd_extension_enabled(&mut self, sxe: bool) -> &mut Self {
        self.bits.set_bit(1, sxe);
        self
    }

    #[doc = "The Advanced SIMD extension instruction enable bit.
If disabled, execution of the Advanced SIMD instruction will trigger an Advanced SIMD extension instruction disable exception (ASXD)."]
    pub fn is_advanced_simd_extension_enabled(&self) -> bool {
        self.bits.get_bit(2)
    }
    #[doc = "The Advanced SIMD extension instruction enable bit.
If disabled, execution of the Advanced SIMD instruction will trigger an Advanced SIMD extension instruction disable exception (ASXD)."]
    pub fn set_advanced_simd_extension_enabled(&mut self, asxe: bool) -> &mut Self {
        self.bits.set_bit(2, asxe);
        self
    }
    #[doc = "The Binary Translation extension instruction enable bit.
If disabled, execution of the Binary Translation instruction will trigger an Binary Translation extension instruction disable exception (BTD)."]
    pub fn is_bin_trans_enabled(&self) -> bool {
        self.bits.get_bit(3)
    }
    #[doc = "The Binary Translation extension instruction enable bit.
If disabled, execution of the Binary Translation instruction will trigger an Binary Translation extension instruction disable exception (BTD)."]
    pub fn set_bin_trans_enabled(&mut self, bte: bool) -> &mut Self {
        self.bits.set_bit(3, bte);
        self
    }
}
