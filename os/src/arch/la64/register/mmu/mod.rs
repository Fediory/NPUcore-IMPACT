#![allow(unused)]

use core::fmt::write;
pub mod asid;
pub mod dmw;
pub mod pgd;
pub mod pwch;
pub mod pwcl;
pub mod stlbps;
pub mod tlbehi;
pub mod tlbelo;
pub mod tlbidx;
pub mod tlbrbadv;
pub mod tlbrehi;
pub mod tlbrelo;
pub mod tlbrentry;
pub mod tlbrera;
pub mod tlbrprmd;
pub mod tlbrsave;

#[derive(Debug, Eq, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(usize)]
pub enum MemoryAccessType {
    StronglyOrderedUnCached = 0,
    CoherentCached = 1,
    WeaklyOrderedUnCached = 2,
}
impl core::fmt::Display for MemoryAccessType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[linkage = "weak"]
pub const VALEN: usize = 48;
#[linkage = "weak"]
pub const PALEN: usize = 48;
