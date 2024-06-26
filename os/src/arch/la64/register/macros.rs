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

macro_rules! impl_define_mem_reg {
    ($mem_reg_ident:ident,$mem_reg_addr:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $mem_reg_ident {
            bits: u32,
        }
        impl_write_mem_reg!($mem_reg_addr, $mem_reg_ident);
        impl_read_mem_reg!($mem_reg_addr, $mem_reg_ident);

        #[allow(unused)]
        impl $mem_reg_ident {
            pub fn get_mem_reg_offset() -> usize {
                $mem_reg_addr
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
        impl bit_field::BitField for $mem_reg_ident {
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

macro_rules! impl_read_mem_reg {
    ($mem_reg_addr:ident,$mem_reg_ident:ident) => {
        impl $mem_reg_ident {
            #[inline(always)]
            #[allow(unused)]
            pub fn read(base: usize) -> $mem_reg_ident {
                $mem_reg_ident {
                    bits: unsafe { (($mem_reg_addr as *mut u32).read_volatile()) },
                }
            }
        }
    };
}

macro_rules! impl_write_mem_reg {
    ($mem_reg_addr:ident,$mem_reg_ident:ident) => {
        impl $mem_reg_ident {
            #[inline(always)]
            #[allow(unused)]
            pub fn write(&mut self) -> &mut Self {
                unsafe {
                    ($mem_reg_addr as *mut u32).write_volatile(self.bits);
                }
                self
            }
        }
    };
}

macro_rules! impl_get_set {
    ($mem_reg_get_ident:ident,$mem_reg_set_ident:ident,$num:literal,$doc:expr) => {
        #[doc = $doc]
        #[inline(always)]
        #[allow(unused)]
        pub fn $mem_reg_get_ident(&self) -> bool {
            self.get_bit($num)
        }

        #[doc = $doc]
        #[inline(always)]
        #[allow(unused)]
        pub fn $mem_reg_set_ident(&mut self, status: bool) -> &mut Self {
            self.set_bit($num, status);
            self
        }
    };
    
    ($mem_reg_get_ident:ident,$mem_reg_set_ident:ident,$range:expr,$doc:expr) => {
        #[doc = $doc]
        #[inline(always)]
        #[allow(unused)]
        pub fn $mem_reg_get_ident(&self) -> usize {
            self.get_bits($range).bits as usize
        }

        #[doc = $doc]
        #[inline(always)]
        #[allow(unused)]
        pub fn $mem_reg_set_ident(&mut self, status: usize) -> &mut Self {
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


