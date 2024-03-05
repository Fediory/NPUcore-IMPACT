macro_rules! impl_read_csr {
    ($csr_number:ident,$csr_ident:ident) => {
        impl $csr_ident {
            #[inline(always)]
            #[allow(unused)]
            pub fn read(base: usize) -> $csr_ident {
                $csr_ident {
                    bits: unsafe { (((base + $csr_number) as *mut u32).read_volatile()) },
                }
            }
        }
    };
}

macro_rules! impl_read_csr_no_offset {
    ($csr_number:ident,$csr_ident:ident) => {
        impl $csr_ident {
            #[inline(always)]
            pub fn read() -> $csr_ident {
                $csr_ident {
                    bits: unsafe { ((($csr_number) as *mut u32).read_volatile()) },
                }
            }
        }
    };
}

macro_rules! impl_write_csr {
    ($csr_number:ident,$csr_ident:ident) => {
        impl $csr_ident {
            #[inline(always)]
            #[allow(unused)]
            pub fn write(&mut self, base: usize) -> &mut Self {
                unsafe {
                    ((base + $csr_number) as *mut u32).write_volatile(self.bits);
                }
                self
            }
        }
    };
}
macro_rules! impl_write_csr_no_offset {
    ($csr_number:ident,$csr_ident:ident) => {
        impl $csr_ident {
            #[inline(always)]
            #[allow(unused)]
            pub fn write(&mut self) -> &mut Self {
                unsafe {
                    (($csr_number) as *mut u32).write_volatile(self.bits);
                }
                self
            }
        }
    };
}
macro_rules! impl_define_mem_reg_no_offset {
    ($csr_ident:ident,$csr_number:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $csr_ident {
            bits: u32,
        }
        impl_write_csr_no_offset!($csr_number, $csr_ident);
        impl_read_csr_no_offset!($csr_number, $csr_ident);
        impl $csr_ident {
            pub fn empty() -> Self {
                Self { bits: 0 }
            }
            pub fn set_value(mut self, bit: u32) -> Self {
                self.bits = bit;
                self
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

macro_rules! impl_define_mem_reg {
    ($csr_ident:ident,$csr_number:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $csr_ident {
            bits: u32,
        }
        impl_write_csr!($csr_number, $csr_ident);
        impl_read_csr!($csr_number, $csr_ident);

        #[allow(unused)]
        impl $csr_ident {
            pub fn get_reg_offset() -> usize {
                $csr_number
            }
            pub fn empty() -> Self {
                Self { bits: 0 }
            }
            #[allow(unused)]
            pub fn set_value(&mut self, bit: u32) -> &mut Self {
                self.bits = bit;
                self
            }
            #[allow(unused)]
            pub fn get_value(&self) -> u32 {
                self.bits
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

macro_rules! impl_define_csr_rd_only {
    ($csr_ident:ident,$csr_number:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $csr_ident {
            bits: u32,
        }
        impl_read_csr!($csr_number, $csr_ident);
        impl $csr_ident {
            #[allow(unused)]
            fn get_bit(&self, bit: usize) -> bool {
                self.bits.get_bit(bit)
            }

            #[allow(unused)]
            fn get_bits<T: core::ops::RangeBounds<usize>>(&self, range: T) -> Self {
                Self {
                    bits: self.bits.get_bits(range),
                }
            }
        }
    };
}

macro_rules! impl_predicate_neg {
    ($csr_ident:ident,$num:literal) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> bool {
            !self.get_bit($num)
        }
    };
}

macro_rules! impl_predicate {
    ($csr_ident:ident,$num:literal) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> bool {
            self.get_bit($num)
        }
    };
}

macro_rules! impl_get_reg_independ {
    ($csr_ident:ident,$ty_name:ident) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> $ty_name {
            $ty_name::read()
        }
    };
}
macro_rules! impl_get_reg {
    ($csr_ident:ident,$ty_name:ident) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> $ty_name {
            $ty_name::read(self.base)
        }
    };
}

macro_rules! impl_get_set_enum {
    ($csr_ident:ident,$csr_set_ident:ident,$range:expr,$enum:ty) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> $enum {
            if let Ok(i) = <$enum>::try_from(self.get_bits($range).bits as usize) {
                i
            } else {
                <$enum>::Reserved
            }
        }
        #[inline(always)]
        pub fn $csr_set_ident(&mut self, status: $enum) -> &mut Self {
            let i: usize = status.try_into().unwrap();
            self.set_bits($range, Self { bits: i as u32 });
            self
        }
    };
}

macro_rules! impl_get_set_field {
    ($csr_ident:ident,$csr_set_ident:ident,$range:expr) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> usize {
            self.get_bits($range).bits as usize
        }

        #[inline(always)]
        pub fn $csr_set_ident(&mut self, status: usize) -> &mut Self {
            self.set_bits(
                $range,
                Self {
                    bits: status as u32,
                },
            );
            self
        }
    };
}

macro_rules! impl_get_set {
    ($csr_ident:ident,$csr_set_ident:ident,$num:literal) => {
        #[inline(always)]
        pub fn $csr_ident(&self) -> bool {
            self.get_bit($num)
        }

        #[inline(always)]
        pub fn $csr_set_ident(&mut self, status: bool) -> &mut Self {
            self.set_bit($num, status);
            self
        }
    };
}
