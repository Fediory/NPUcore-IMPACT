macro_rules! impl_read_csr {
    ($csr_number:literal,$csr_ident:ident) => {
        impl $csr_ident {
            #[inline(always)]
            pub fn read() -> $csr_ident {
                $csr_ident {
                    bits: unsafe {
                        let bits:usize;
                        core::arch::asm!("csrrd {},{}", out(reg) bits, const $csr_number);
                        bits
                    },
                }
            }
        }
    };
}

macro_rules! impl_write_csr {
    ($csr_number:literal,$csr_ident:ident) => {
        impl $csr_ident {
            #[inline(always)]
            pub fn write(self) {
                unsafe {
                    core::arch::asm!("csrwr {},{}", in(reg) self.bits, const $csr_number);
                }
            }
        }
    };
}
macro_rules! impl_define_csr {
    ($csr_ident:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $csr_ident {
            bits: usize,
        }
        impl $csr_ident {
            pub fn empty() -> Self {
                Self { bits: 0 }
            }
            pub fn from(bits: usize) -> Self {
                Self { bits }
            }
        }
        impl bit_field::BitField for $csr_ident {
            const BIT_LENGTH: usize = usize::BIT_LENGTH;

            fn get_bit(&self, bit: usize) -> bool {
                self.bits.get_bit(bit)
            }

            fn get_bits<T: core::ops::RangeBounds<usize>>(&self, range: T) -> Self {
                Self {
                    bits: self.bits.get_bits(range),
                }
            }

            fn set_bit(&mut self, bit: usize, value: bool) -> &mut Self {
                self.bits.set_bit(bit, value);
                self
            }

            fn set_bits<T: core::ops::RangeBounds<usize>>(
                &mut self,
                range: T,
                value: Self,
            ) -> &mut Self {
                self.bits.set_bits(range, value.bits);
                self
            }
        }
    };
}
