use super::{tlb::tlb_invalidate, tlb_global_invalidate};
use crate::{
    config::{
        MEMORY_HIGH_BASE, MEMORY_HIGH_BASE_VPN, MEMORY_SIZE, PAGE_SIZE, PAGE_SIZE_BITS, PALEN,
        VA_MASK, VPN_SEG_MASK,DIRTY_WIDTH,
    },
    mm::{address::*, frame_alloc, FrameTracker, MapPermission, PageTable},
};
use _core::convert::TryFrom;
use alloc::{sync::Arc, vec::Vec};
use bitflags::*;
use log::trace;
static mut DIRTY: [bool; DIRTY_WIDTH] = [false; DIRTY_WIDTH];
use super::register::MemoryAccessType;

bitflags! {
    /// Page Table Entry flags
    pub struct LAPTEFlagBits: usize {
        /// Valid Bit
        const V = 1 << 0;
        /// Dirty Bit, true if it is modified.
        const D = 1 << 1;
        /// Privilege Level field
        const PLV0 = 0;
        const PLV1 = 1 << 2;
        const PLV2 = 2 << 2;
        const PLV3 = 3 << 2;
        /// Memory Access Type: Strongly-ordered UnCached (SUC)
        const MAT_SUC = 0 << 4;
        /// Memory Access Type: Coherent Cached (CC)
        const MAT_CC = 1 << 4;
        /// Memory Access Type: Weakly-ordered UnCached (WUC)
        const MAT_WUC = 2 << 4;
        /// Global Bit (Basic PTE)
        const G = 1 << 6;
        /// Physical Bit, whether the physical page exists
        const P = 1 << 7;
        /// Writable Bit
        const W = 1 << 8;

        /// Not Readable Bit
        const NR = 1 << (usize::BITS-3); // 61
        /// Executable Bit
        const NX = 1 << (usize::BITS-2); // 62
        /// Restricted Privilege LeVel enable (RPLV) for the page table.
        /// When RPLV=0, the page table entry can be accessed by any program whose privilege level is not lower than PLV;
        /// when RPLV=1, the page table entry can only be accessed by programs whose privilege level is equal to PLV.
        const RPLV = 1 << (usize::BITS-1); // 63

    }
}
#[allow(unused)]
impl LAPTEFlagBits {
    pub fn get_memory_access_type(&self) -> MemoryAccessType {
        MemoryAccessType::try_from((self.bits) >> 4 & 3).unwrap()
    }
    pub fn set_memory_access_type(&mut self, mat: MemoryAccessType) {
        match mat {
            MemoryAccessType::StronglyOrderedUnCached => self.set(Self::MAT_SUC, true),
            MemoryAccessType::CoherentCached => self.set(Self::MAT_CC, true),
            MemoryAccessType::WeaklyOrderedUnCached => self.set(Self::MAT_WUC, true),
        }
    }
    pub fn get_privilege_level(&self) -> usize {
        (self.bits >> 2) & 3 as usize
    }
}

/// Page Table Entry
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LAFlexPageTableEntry {
    pub bits: usize,
}
impl LAFlexPageTableEntry {
    const PPN_MASK: usize = ((1usize << PALEN) - 1) << 12;
    #[inline(always)]
    pub fn new(ppn: PhysPageNum, flags: LAPTEFlagBits) -> Self {
        let bits: usize = ((ppn.0 << 12) & Self::PPN_MASK) | flags.bits as usize;
        LAFlexPageTableEntry { bits }
    }
    #[allow(unused)]
    #[inline(always)]
    pub fn empty() -> Self {
        LAFlexPageTableEntry {
            bits: (LAPTEFlagBits::NR & LAPTEFlagBits::NX).bits(),
        }
    }
    #[inline(always)]
    pub fn ppn(&self) -> PhysPageNum {
        ((self.bits & Self::PPN_MASK) >> 12).into()
    }
    #[inline(always)]
    pub fn set_ppn(&mut self, ppn: PhysPageNum) {
        self.bits = (self.bits & !Self::PPN_MASK) | ((ppn.0 << 12) & Self::PPN_MASK)
    }
    #[inline(always)]
    pub fn flags(&self) -> LAPTEFlagBits {
        LAPTEFlagBits {
            bits: self.bits & !Self::PPN_MASK,
        }
    }
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.flags().contains(LAPTEFlagBits::V)
    }
    pub fn set_dirty(&mut self) {
        self.bits |= LAPTEFlagBits::D.bits;
    }
    #[inline(always)]
    pub fn is_dirty(&self) -> bool {
        self.flags().contains(LAPTEFlagBits::D)
    }
    #[inline(always)]
    pub fn writable(&self) -> bool {
        self.flags().contains(LAPTEFlagBits::W)
    }
    #[inline(always)]
    pub fn readable(&self) -> bool {
        !self.flags().contains(LAPTEFlagBits::NR)
    }
    #[inline(always)]
    pub fn executable(&self) -> bool {
        !self.flags().contains(LAPTEFlagBits::NX)
    }
    /// LA hasn't had access bit so far. So this function is left empty.
    #[inline(always)]
    pub fn clear_access(&mut self) {}

    #[inline(always)]
    pub fn clear_dirty(&mut self) {
        self.bits &= !(LAPTEFlagBits::D.bits() as usize);
    }
    #[inline(always)]
    pub fn revoke_write(&mut self) {
        self.bits &= !(LAPTEFlagBits::W.bits() as usize);
    }
    #[inline(always)]
    pub fn revoke_read(&mut self) {
        self.bits |= LAPTEFlagBits::NR.bits();
    }
    #[inline(always)]
    pub fn revoke_execute(&mut self) {
        self.bits |= LAPTEFlagBits::NX.bits() as usize;
    }
    #[inline(always)]
    pub fn set_permission(&mut self, flags: MapPermission) {
        if flags.contains(MapPermission::R) {
            self.bits &= !LAPTEFlagBits::NR.bits();
        } else {
            self.revoke_read()
        }
        if flags.contains(MapPermission::X) {
            self.bits &= !LAPTEFlagBits::NX.bits();
        } else {
            self.revoke_execute();
        }
        if flags.contains(MapPermission::W) {
            self.bits |= LAPTEFlagBits::W.bits();
        } else {
            self.revoke_write();
        }
        if flags.contains(MapPermission::U) {
            self.bits |= (LAPTEFlagBits::PLV3).bits();
            self.bits &= !LAPTEFlagBits::RPLV.bits();
        } else {
            self.bits &= !LAPTEFlagBits::PLV3.bits();
            self.bits |= LAPTEFlagBits::RPLV.bits();
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct LAPTRoot(pub usize);
impl From<usize> for LAPTRoot {
    fn from(v: usize) -> Self {
        Self(v)
    }
}
pub struct LAFlexPageTable {
    root_ppn: LAPTRoot,
    frames: Vec<Arc<FrameTracker>>,
}

/// Assume that it won't encounter oom when creating/mapping.
impl LAFlexPageTable {
    fn is_ident_map(&self, vpn: VirtPageNum) -> bool {
        self.is_kernel_pt() && (vpn.0 & VPN_SEG_MASK == MEMORY_HIGH_BASE_VPN)
    }
    fn is_kernel_pt(&self) -> bool {
        (self.token() as u32) == 0
    }
    fn get_root_ppn(&self) -> PhysPageNum {
        if self.is_kernel_pt() {
            PhysPageNum(self.token() >> 32)
        } else {
            PhysPageNum(self.token())
        }
    }
    /// Find the page in the page table, creating the page on the way if not exists.
    /// Note: It does NOT create the terminal node. The caller must verify its validity and create according to his own needs.
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut LAFlexPageTableEntry> {
        //trace!("[find_pte_create] {:?}", vpn);
        let idxs = vpn.indexes::<3>();
        //log::trace!("[find_pte_create] idxs:{:?}", idxs);
        let mut ppn = self.get_root_ppn();
        let mut pte = &mut ppn.get_pte_array::<LAFlexPageTableEntry>()[idxs[0]];

        if !pte.is_valid() {
            let frame = frame_alloc().unwrap();
            *pte = LAFlexPageTableEntry::new(frame.ppn, LAPTEFlagBits::V);
            self.frames.push(frame);
        }
        ppn = PhysAddr::from((pte.ppn().0 << 12) | MEMORY_HIGH_BASE).floor();
        pte = &mut ppn.get_pte_array::<LAFlexPageTableEntry>()[idxs[1]];
        if !pte.is_valid() {
            let frame = frame_alloc().unwrap();
            *pte = LAFlexPageTableEntry::new(frame.ppn, LAPTEFlagBits::V);
            self.frames.push(frame);
        }
        ppn = PhysAddr::from((pte.ppn().0 << 12) | MEMORY_HIGH_BASE).floor();
        pte = &mut ppn.get_pte_array::<LAFlexPageTableEntry>()[idxs[2]];
        Some(pte)
    }
    /// Find and return reference the page table entry denoted by `vpn`, `None` if not found or invalid.
    fn find_pte_refmut(&self, vpn: VirtPageNum) -> Option<&mut LAFlexPageTableEntry> {
        //trace!("[find_pte_refmut] {:?}", vpn);
        let idxs = vpn.indexes::<3>();
        let mut ppn = self.get_root_ppn();
        let mut pte = &mut ppn.get_pte_array::<LAFlexPageTableEntry>()[idxs[0]];
        if !pte.is_valid() {
            return None;
        }
        ppn = PhysAddr::from((pte.ppn().0 << 12) | MEMORY_HIGH_BASE).floor();
        pte = &mut ppn.get_pte_array::<LAFlexPageTableEntry>()[idxs[1]];
        if !pte.is_valid() {
            return None;
        }
        ppn = PhysAddr::from((pte.ppn().0 << 12) | MEMORY_HIGH_BASE).floor();
        pte = &mut ppn.get_pte_array::<LAFlexPageTableEntry>()[idxs[2]];
        if pte.is_valid() {
            Some(pte)
        } else {
            None
        }
    }
    /// Find the page table entry denoted by vpn, returning Some(&_) if found or None if not.
    #[inline(always)]
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&LAFlexPageTableEntry> {
        //trace!("[find_pte(as refmut)] {:?}", vpn);
        self.find_pte_refmut(vpn).map(|i| &*i)
    }
    pub fn set_dirty_bit(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        tlb_invalidate();
        if self.is_ident_map(vpn) {
            unsafe {
                DIRTY[vpn.0 & VA_MASK] = true;
            }
            return Ok(());
        }
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.set_dirty();
            Ok(())
        } else {
            Err(())
        }
    }
}
/// Assume that it won't encounter oom when creating/mapping.
impl PageTable for LAFlexPageTable {
    fn new_kern_space() -> Self
    where
        Self: Sized,
    {
        let frame = frame_alloc().unwrap();
        trace!("root ppn:{:?}", frame);
        LAFlexPageTable {
            root_ppn: LAPTRoot(frame.ppn.0 << 32),
            frames: {
                let mut vec = Vec::with_capacity(256);
                vec.push(frame);
                vec
            },
        }
    }
    fn new() -> Self {
        let frame = frame_alloc().unwrap();
        trace!("root ppn:{:?}", frame);
        LAFlexPageTable {
            root_ppn: LAPTRoot(frame.ppn.0),
            frames: {
                let mut vec = Vec::with_capacity(256);
                vec.push(frame);
                vec
            },
        }
    }
    /// Create an empty page table from `satp`
    /// # Argument
    /// * `satp` Supervisor Address Translation & Protection reg. that points to the physical page containing the root page.
    fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: LAPTRoot::from(satp),
            frames: Vec::new(),
        }
    }
    /// Predicate for the valid bit.
    fn is_mapped(&mut self, vpn: VirtPageNum) -> bool {
        let ret = {
            if self.is_ident_map(vpn) {
                true
            } else {
                if let Some(i) = self.find_pte(vpn) {
                    if i.is_valid() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };
        ret
    }
    fn map_identical(&mut self, _vpn: VirtPageNum, _ppn: PhysPageNum, _flags: MapPermission) {}
    /// Find the page in the page table, creating the page on the way if not exists.
    /// Note: It does NOT create the terminal node. The caller must verify its validity and create according to his own needs.
    /// Map the `vpn` to `ppn` with the `flags`.
    /// # Note
    /// Allocation should be done elsewhere.
    /// # Exceptions
    /// Panics if the `vpn` is mapped.
    #[allow(unused)]
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: MapPermission) {
        let pte = self.find_pte_create(vpn).unwrap();
        //log::trace!("[laflex::map] vpn: {:?}, ppn:{:?}", vpn, ppn);
        debug_assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        let mut flag = LAPTEFlagBits::V | LAPTEFlagBits::MAT_CC;
        if !flags.contains(MapPermission::R) {
            flag |= LAPTEFlagBits::NR;
        }
        if !flags.contains(MapPermission::X) {
            flag |= LAPTEFlagBits::NX;
        }
        if flags.contains(MapPermission::W) {
            flag |= LAPTEFlagBits::W;
        }
        if flags.contains(MapPermission::U) {
            flag |= LAPTEFlagBits::PLV3;
        }
        //flag |= LAPTEFlagBits::D;
        let pte_new = LAFlexPageTableEntry::new(ppn, flag);
        //log::trace!("[laflex::map] pre_wr");
        *pte = pte_new;
    }
    #[allow(unused)]
    /// Unmap the `vpn` to `ppn` with the `flags`.
    /// # Exceptions
    /// Panics if the `vpn` is NOT mapped (invalid).
    fn unmap(&mut self, vpn: VirtPageNum) {
        //tlb_invalidate();
        let pte = self.find_pte_refmut(vpn).unwrap(); // was `self.find_creat_pte(vpn).unwrap()`;
        debug_assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = LAFlexPageTableEntry { bits: 0 };
    }
    /// Translate the `vpn` into its corresponding `Some(PageTableEntry)` if exists
    /// `None` is returned if nothing is found.
    fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        // This is not the same map as we defined just now...
        // It is the map for func. programming.
        self.find_pte(vpn).map(|pte| pte.ppn())
    }
    /// Translate the virtual address into its corresponding `PhysAddr` if mapped in current page table.
    /// `None` is returned if nothing is found.
    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }
    fn block_and_ret_mut(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.clear_dirty();
            pte.revoke_write();
            Some(pte.ppn())
        } else {
            None
        }
    }
    /// Return the physical token to current page.
    /// NOTE: NEVER use this token to fill a PGD!
    /// IT is NOT (necessarily) a PPN!
    fn token(&self) -> usize {
        self.root_ppn.0
    }
    fn revoke_read(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.revoke_read();
            Ok(())
        } else {
            Err(())
        }
    }
    fn revoke_write(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.revoke_write();
            Ok(())
        } else {
            Err(())
        }
    }
    fn revoke_execute(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.revoke_execute();
            Ok(())
        } else {
            Err(())
        }
    }
    fn set_ppn(&mut self, vpn: VirtPageNum, ppn: PhysPageNum) -> Result<(), ()> {
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.set_ppn(ppn);
            Ok(())
        } else {
            Err(())
        }
    }
    fn set_pte_flags(&mut self, vpn: VirtPageNum, flags: MapPermission) -> Result<(), ()> {
        //tlb_invalidate();
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.set_permission(flags);
            Ok(())
        } else {
            Err(())
        }
    }
    fn clear_access_bit(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        tlb_invalidate();
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.clear_access();
            Ok(())
        } else {
            Err(())
        }
    }
    fn clear_dirty_bit(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        tlb_invalidate();
        if self.is_ident_map(vpn) {
            unsafe {
                DIRTY[vpn.0 & VA_MASK] = false;
            }
            return Ok(());
        }
        if let Some(pte) = self.find_pte_refmut(vpn) {
            pte.clear_dirty();
            Ok(())
        } else {
            Err(())
        }
    }
    fn activate(&self) {
        tlb_global_invalidate();
        if self.is_kernel_pt() {
            super::register::PGDH::from(self.get_root_ppn().0 << PAGE_SIZE_BITS).write();
        } else {
            super::register::PGDL::from(self.get_root_ppn().0 << PAGE_SIZE_BITS).write();
        }
    }
    fn is_valid(&self, vpn: VirtPageNum) -> Option<bool> {
        self.find_pte(vpn).map(|pte| pte.is_valid())
    }
    fn is_dirty(&self, vpn: VirtPageNum) -> Option<bool> {
        if self.is_ident_map(vpn) {
            Some(unsafe { DIRTY[vpn.0 & VA_MASK] })
        } else {
            self.find_pte(vpn).map(|pte| pte.is_dirty())
        }
    }
    fn readable(&self, vpn: VirtPageNum) -> Option<bool> {
        self.find_pte(vpn).map(|pte| pte.readable())
    }
    fn writable(&self, vpn: VirtPageNum) -> Option<bool> {
        self.find_pte(vpn).map(|pte| pte.writable())
    }
    fn executable(&self, vpn: VirtPageNum) -> Option<bool> {
        self.find_pte(vpn).map(|pte| pte.executable())
    }

    fn unmap_identical(&mut self, vpn: VirtPageNum) {
        self.unmap(vpn)
    }
}
