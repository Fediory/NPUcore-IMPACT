//use super::Sv39PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

/// Debug formatter for VirtPageNum
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VPN")
            .field(&format_args!("{:#X}", self.0))
            .finish()
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

impl VirtPageNum {
    pub fn start_addr(&self) -> VirtAddr {
        VirtAddr::from(self.0 << PAGE_SIZE_BITS)
    }
    pub fn offset(&self, offset: usize) -> VirtAddr {
        VirtAddr::from((self.0 << PAGE_SIZE_BITS) + offset)
    }
    pub fn indexes<const T: usize>(&self) -> [usize; T] {
        let mut vpn = self.0;
        let mut idx = [0usize; T];
        for i in (0..T).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}