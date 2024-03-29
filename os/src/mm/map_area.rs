use core::fmt::Debug;

use super::page_table::PageTable;
#[cfg(feature = "zram")]
use super::zram::{ZramTracker, ZRAM_DEVICE};
use super::MemoryError;
use super::VPNRange;
use super::KERNEL_SPACE;
use super::{frame_alloc, FrameTracker};
use super::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::fs::file_trait::File;
#[cfg(feature = "swap")]
use crate::fs::swap::{SwapTracker, SWAP_DEVICE};
use crate::fs::SeekWhence;
use crate::mm::frame_allocator::frame_alloc_uninit;

#[cfg(feature = "oom_handler")]
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use log::{error, trace, warn};
#[cfg(feature = "oom_handler")]
#[derive(Clone, Debug)]
pub enum Frame {
    InMemory(Arc<FrameTracker>),
    Compressed(Arc<ZramTracker>),
    SwappedOut(Arc<SwapTracker>),
    Unallocated,
}

#[cfg(not(feature = "oom_handler"))]
#[derive(Clone, Debug)]
pub enum Frame {
    InMemory(Arc<FrameTracker>),
    Unallocated,
}

impl Frame {
    pub fn insert_in_memory(
        &mut self,
        frame_tracker: Arc<FrameTracker>,
    ) -> Result<(), MemoryError> {
        match self {
            Frame::Unallocated => {
                *self = Frame::InMemory(frame_tracker);
                Ok(())
            }
            _ => Err(MemoryError::AlreadyAllocated),
        }
    }
    pub fn take_in_memory(&mut self) -> Option<Arc<FrameTracker>> {
        match self {
            Frame::InMemory(frame_ref) => {
                // avoid implement trait 'Copy'
                let frame = unsafe { core::ptr::read(frame_ref) };
                // avoid drop
                unsafe { core::ptr::write(self, Frame::Unallocated) };
                Some(frame)
            }
            _ => None,
        }
    }
    #[cfg(feature = "oom_handler")]
    pub fn swap_out(&mut self) -> Result<usize, MemoryError> {
        match self {
            Frame::InMemory(frame_ref) => {
                if Arc::strong_count(frame_ref) == 1 {
                    let swap_tracker = SWAP_DEVICE.lock().write(frame_ref.ppn.get_bytes_array());
                    let swap_id = swap_tracker.0;
                    // frame_tracker should be dropped
                    *self = Frame::SwappedOut(swap_tracker);
                    Ok(swap_id)
                } else {
                    Err(MemoryError::SharedPage)
                }
            }
            _ => Err(MemoryError::NotInMemory),
        }
    }
    /// # Warning
    /// This function do not check reference count of frame,
    /// So it's possible that some pages was write to external storage, but no page is released.
    #[cfg(feature = "oom_handler")]
    pub fn force_swap_out(&mut self) -> Result<usize, MemoryError> {
        match self {
            Frame::InMemory(frame_ref) => {
                let swap_tracker = SWAP_DEVICE.lock().write(frame_ref.ppn.get_bytes_array());
                let swap_id = swap_tracker.0;
                // frame_tracker should be dropped
                *self = Frame::SwappedOut(swap_tracker);
                Ok(swap_id)
            }
            _ => Err(MemoryError::NotInMemory),
        }
    }
    #[cfg(feature = "oom_handler")]
    pub fn swap_in(&mut self) -> Result<PhysPageNum, MemoryError> {
        match self {
            Frame::SwappedOut(swap_tracker) => {
                let frame = frame_alloc().unwrap();
                let ppn = frame.ppn;
                SWAP_DEVICE
                    .lock()
                    .read(swap_tracker.0, ppn.get_bytes_array());
                *self = Frame::InMemory(frame);
                Ok(ppn)
            }
            _ => Err(MemoryError::NotSwappedOut),
        }
    }
    #[cfg(feature = "oom_handler")]
    pub fn zip(&mut self) -> Result<usize, MemoryError> {
        match self {
            Frame::InMemory(frame_ref) => {
                if Arc::strong_count(frame_ref) == 1 {
                    if let Ok(zram_tracker) =
                        ZRAM_DEVICE.lock().write(frame_ref.ppn.get_bytes_array())
                    {
                        let zram_id = zram_tracker.0;
                        // frame_tracker should be dropped
                        *self = Frame::Compressed(zram_tracker);
                        Ok(zram_id)
                    } else {
                        Err(MemoryError::ZramIsFull)
                    }
                } else {
                    Err(MemoryError::SharedPage)
                }
            }
            _ => Err(MemoryError::NotInMemory),
        }
    }
    #[cfg(feature = "oom_handler")]
    pub fn unzip(&mut self) -> Result<PhysPageNum, MemoryError> {
        match self {
            Frame::Compressed(zram_tracker) => {
                let frame = frame_alloc().unwrap();
                let ppn = frame.ppn;
                ZRAM_DEVICE
                    .lock()
                    .read(zram_tracker.0, ppn.get_bytes_array())
                    .unwrap();
                *self = Frame::InMemory(frame);
                Ok(ppn)
            }
            _ => Err(MemoryError::NotCompressed),
        }
    }
}

#[derive(Clone)]
pub struct LinearMap {
    pub vpn_range: VPNRange,
    pub frames: Vec<Frame>,
    #[cfg(feature = "oom_handler")]
    pub active: VecDeque<u16>,
    #[cfg(feature = "oom_handler")]
    pub compressed: usize,
    #[cfg(feature = "oom_handler")]
    pub swapped: usize,
}
impl Debug for LinearMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "oom_handler")]
        return f
            .debug_struct("LinearMap")
            .field("vpn_range", &self.vpn_range)
            .field("active", &self.active.len())
            .field("compressed", &self.compressed)
            .field("swapped", &self.swapped)
            .finish();
        #[cfg(not(feature = "oom_handler"))]
        return f
            .debug_struct("LinearMap")
            .field("vpn_range", &self.vpn_range)
            .field("frames", &self.frames)
            .finish();
    }
}
impl LinearMap {
    pub fn new(vpn_range: VPNRange) -> Self {
        let len = vpn_range.get_end().0 - vpn_range.get_start().0;
        let mut new_dict = Self {
            vpn_range,
            frames: Vec::with_capacity(len),
            #[cfg(feature = "oom_handler")]
            active: VecDeque::new(),
            #[cfg(feature = "oom_handler")]
            compressed: 0,
            #[cfg(feature = "oom_handler")]
            swapped: 0,
        };
        new_dict.frames.resize(len, Frame::Unallocated);
        new_dict
    }
    pub fn get_mut(&mut self, key: &VirtPageNum) -> &mut Frame {
        &mut self.frames[key.0 - self.vpn_range.get_start().0]
    }
    /// # Warning
    /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn get_in_memory(&self, key: &VirtPageNum) -> Option<&Arc<FrameTracker>> {
        match &self.frames[key.0 - self.vpn_range.get_start().0] {
            Frame::InMemory(tracker) => Some(tracker),
            _ => None,
        }
    }
    /// # Warning
    /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn alloc_in_memory(&mut self, key: VirtPageNum, value: Arc<FrameTracker>) {
        let idx = key.0 - self.vpn_range.get_start().0;
        #[cfg(feature = "oom_handler")]
        self.active.push_back(idx as u16);
        self.frames[idx].insert_in_memory(value).unwrap()
    }
    /// # Warning
    /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn remove_in_memory(&mut self, key: &VirtPageNum) -> Option<Arc<FrameTracker>> {
        let idx = key.0 - self.vpn_range.get_start().0;
        #[cfg(feature = "oom_handler")]
        self.active.retain(|&elem| elem as usize != idx);
        self.frames[idx].take_in_memory()
    }
    // /// # Warning
    // /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn set_start(&mut self, new_vpn_start: VirtPageNum) -> Result<(), ()> {
        let vpn_start = self.vpn_range.get_start();
        let vpn_end = self.vpn_range.get_end();
        if new_vpn_start > vpn_end {
            return Err(());
        }
        self.vpn_range = VPNRange::new(new_vpn_start, vpn_end);
        if new_vpn_start < vpn_start {
            self.frames.rotate_left(vpn_start.0 - new_vpn_start.0);
        } else {
            self.frames.rotate_left(new_vpn_start.0 - vpn_start.0);
        }
        self.frames
            .resize(vpn_end.0 - new_vpn_start.0, Frame::Unallocated);
        Ok(())
    }
    pub fn set_end(&mut self, new_vpn_end: VirtPageNum) -> Result<(), ()> {
        let vpn_start = self.vpn_range.get_start();
        self.vpn_range = VPNRange::new(vpn_start, new_vpn_end);
        if vpn_start > new_vpn_end {
            return Err(());
        }
        self.frames
            .resize(new_vpn_end.0 - vpn_start.0, Frame::Unallocated);
        Ok(())
    }
    #[inline(always)]
    pub fn into_two(&mut self, cut: VirtPageNum) -> Result<Self, ()> {
        let vpn_start = self.vpn_range.get_start();
        let vpn_end = self.vpn_range.get_end();
        if cut <= vpn_start || cut >= vpn_end {
            return Err(());
        }
        let second_frames = self.frames.split_off(cut.0 - vpn_start.0);

        #[cfg(feature = "oom_handler")]
        let ((first_active, second_active), (first_compressed, first_swapped)) = (
            LinearMap::split_active_into_two(&self.active, cut.0 - vpn_start.0),
            self.count_compressed_and_swapped(0, cut.0 - vpn_start.0),
        );

        let second = LinearMap {
            vpn_range: VPNRange::new(cut, vpn_end),
            frames: second_frames,
            #[cfg(feature = "oom_handler")]
            active: second_active,
            #[cfg(feature = "oom_handler")]
            compressed: self.compressed - first_compressed,
            #[cfg(feature = "oom_handler")]
            swapped: self.swapped - first_swapped,
        };

        self.vpn_range = VPNRange::new(vpn_start, cut);

        #[cfg(feature = "oom_handler")]
        {
            self.active = first_active;
            self.compressed = first_compressed;
            self.swapped = first_swapped;
        }
        Ok(second)
    }
    pub fn into_three(
        &mut self,
        first_cut: VirtPageNum,
        second_cut: VirtPageNum,
    ) -> Result<(Self, Self), ()> {
        if let Ok(mut second) = self.into_two(first_cut) {
            if let Ok(third) = second.into_two(second_cut) {
                return Ok((second, third));
            }
        }
        return Err(());
    }
}
#[cfg(feature = "oom_handler")]
impl LinearMap {
    fn count_compressed_and_swapped(&self, start: usize, end: usize) -> (usize, usize) {
        if self.compressed == 0 && self.swapped == 0 {
            (0, 0)
        } else {
            self.frames[start..end].iter().fold(
                (0, 0),
                |(compressed, swapped), frame| match frame {
                    Frame::Compressed(_) => (compressed + 1, swapped),
                    Frame::SwappedOut(_) => (compressed, swapped + 1),
                    _ => (compressed, swapped),
                },
            )
        }
    }
    fn split_active_into_two(
        active: &VecDeque<u16>,
        cut_idx: usize,
    ) -> (VecDeque<u16>, VecDeque<u16>) {
        if active.is_empty() {
            (VecDeque::new(), VecDeque::new())
        } else {
            active.iter().fold(
                (VecDeque::new(), VecDeque::new()),
                |(mut first_active, mut second_active), &idx| {
                    if (idx as usize) < cut_idx {
                        first_active.push_back(idx);
                    } else {
                        second_active.push_back(idx - cut_idx as u16);
                    }
                    (first_active, second_active)
                },
            )
        }
    }
    #[allow(unused)]
    fn split_active_into_three(
        active: &VecDeque<u16>,
        first_cut_idx: usize,
        second_cut_idx: usize,
    ) -> (VecDeque<u16>, VecDeque<u16>, VecDeque<u16>) {
        assert!(first_cut_idx < second_cut_idx);
        if active.is_empty() {
            (VecDeque::new(), VecDeque::new(), VecDeque::new())
        } else {
            active.iter().fold(
                (VecDeque::new(), VecDeque::new(), VecDeque::new()),
                |(mut first_active, mut second_active, mut third_active), &idx| {
                    if (idx as usize) < first_cut_idx {
                        first_active.push_back(idx);
                    } else if (idx as usize) < second_cut_idx {
                        second_active.push_back(idx - first_cut_idx as u16);
                    } else {
                        third_active.push_back(idx - second_cut_idx as u16)
                    }
                    (first_active, second_active, third_active)
                },
            )
        }
    }
}
impl Debug for MapArea {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MapArea")
            .field("interval", &self.inner)
            .field("map_type", &self.map_type)
            .field("map_perm", &self.map_perm)
            .field(
                "map_file",
                &if self.map_file.is_some() { "yes" } else { "no" },
            )
            .finish()
    }
}
#[derive(Clone)]
/// Map area for different segments or a chunk of memory for memory mapped file access.
pub struct MapArea {
    /// Range of the mapped virtual page numbers.
    /// Page aligned.
    /// Map physical page frame tracker to virtual pages for RAII & lookup.
    pub inner: LinearMap,
    /// Direct or framed(virtual) mapping?
    map_type: MapType,
    /// Permissions which are the or of RWXU, where U stands for user.
    pub map_perm: MapPermission,
    pub map_file: Option<Arc<dyn File>>,
}

impl MapArea {
    /// Construct a new segment without without allocating memory
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
        map_file: Option<Arc<dyn File>>,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        trace!(
            "[MapArea new] start_vpn:{:X}; end_vpn:{:X}; map_perm:{:?}",
            start_vpn.0,
            end_vpn.0,
            map_perm
        );
        Self {
            inner: LinearMap::new(VPNRange::new(start_vpn, end_vpn)),
            map_type,
            map_perm,
            map_file,
        }
    }
    /// Copier, but the physical pages are not allocated,
    /// thus leaving `data_frames` empty.
    pub fn from_another(another: &MapArea) -> Self {
        Self {
            inner: LinearMap::new(VPNRange::new(
                another.inner.vpn_range.get_start(),
                another.inner.vpn_range.get_end(),
            )),
            map_type: another.map_type,
            map_perm: another.map_perm,
            map_file: another.map_file.clone(),
        }
    }
    /// Create `MapArea` from `Vec<Arc<FrameTracker>>`. This function should only be used to
    /// generate a `MapArea` in `KERNEL_SPACE`. \
    /// # NOTE
    /// `start_vpn` will be set to `start_va.floor()`,
    /// `end_vpn` will be set to `start_vpn + frames.len()`,
    /// `map_file` will be set to `None`.
    #[cfg(feature = "oom_handler")]
    pub fn from_existing_frame(
        start_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
        frames: Vec<Frame>,
    ) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = VirtPageNum::from(start_vpn.0 + frames.len());
        Self {
            inner: LinearMap {
                vpn_range: VPNRange::new(start_vpn, end_vpn),
                frames,
                // Unsafe if this `MapArea` is inserted to somewhere except `KERNEL_SPACE`.
                active: VecDeque::new(),
                compressed: 0,
                swapped: 0,
            },
            map_type,
            map_perm,
            map_file: None,
        }
    }
    #[cfg(not(feature = "oom_handler"))]
    pub fn from_existing_frame(
        start_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
        frames: Vec<Frame>,
    ) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = VirtPageNum::from(start_vpn.0 + frames.len());
        Self {
            inner: LinearMap {
                vpn_range: VPNRange::new(start_vpn, end_vpn),
                frames,
            },
            map_type,
            map_perm,
            map_file: None,
        }
    }
    /// Map an included page in current area.
    /// If the `map_type` is `Framed`, then physical pages shall be allocated by this function.
    /// Otherwise, where `map_type` is `Identical`,
    /// the virtual page will be mapped directly to the physical page with an identical address to the page.
    /// # Note
    /// Vpn should be in this map area, but the check is not enforced in this function!
    pub fn map_one<T: PageTable>(
        &mut self,
        page_table: &mut T,
        vpn: VirtPageNum,
    ) -> Result<PhysPageNum, (MemoryError, VirtPageNum)> {
        if self.map_type == MapType::Identical || !page_table.is_mapped(vpn) {
            //if not mapped
            Ok(self.map_one_unchecked(page_table, vpn))
        } else {
            //mapped
            Err((MemoryError::AlreadyMapped, vpn))
        }
    }
    pub fn map_one_unchecked<T: PageTable>(
        &mut self,
        page_table: &mut T,
        vpn: VirtPageNum,
    ) -> PhysPageNum {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
                page_table.map_identical(vpn, ppn, self.map_perm);
            }
            MapType::Framed => {
                let frame = unsafe { frame_alloc_uninit().unwrap() };
                ppn = frame.ppn;
                self.inner.alloc_in_memory(vpn, frame);
                page_table.map(vpn, ppn, self.map_perm);
            }
        }
        ppn
    }
    pub fn map_one_zeroed_unchecked<T: PageTable>(
        &mut self,
        page_table: &mut T,
        vpn: VirtPageNum,
    ) -> PhysPageNum {
        let frame = frame_alloc().unwrap();
        let ppn = frame.ppn;
        self.inner.alloc_in_memory(vpn, frame);
        page_table.map(vpn, ppn, self.map_perm);
        ppn
    }
    /// Unmap a page in current area.
    /// If it is framed, then the physical pages will be removed from the `data_frames` Btree.
    /// This is unnecessary if the area is directly mapped.
    /// # Note
    /// Vpn should be in this map area, but the check is not enforced in this function!
    pub fn unmap_one<T: PageTable>(
        &mut self,
        page_table: &mut T,
        vpn: VirtPageNum,
    ) -> Result<(), MemoryError> {
        if !page_table.is_mapped(vpn) {
            return Err(MemoryError::NotMapped);
        }
        match self.map_type {
            MapType::Framed => {
                self.inner.remove_in_memory(&vpn);
                page_table.unmap(vpn);
            }
            _ => {}
        }
        Ok(())
    }
    /// Map the same area in `self` from `src_page_table` to `dst_page_table`, sharing the same physical address.
    /// Convert map areas to physical pages.
    /// # Of Course...
    /// Since the area is shared, the pages have been allocated.
    /// # Argument
    /// `dst_page_table`: The destination to be mapped into.
    /// `src_page_table`: The source to be mapped from. This is also the page table where `self` should be included.
    pub fn map_from_existing_page_table<T: PageTable>(
        &mut self,
        dst_page_table: &mut T,
        src_page_table: &mut T,
    ) -> Result<(), ()> {
        let map_perm = self.map_perm.difference(MapPermission::W);
        for vpn in self.inner.vpn_range {
            if let Some(ppn) = src_page_table.block_and_ret_mut(vpn) {
                if !dst_page_table.is_mapped(vpn) {
                    dst_page_table.map(vpn, ppn, map_perm);
                } else {
                    return Err(());
                }
            }
        }
        Ok(())
    }
    pub fn get_start<T: PageTable>(&self) -> VirtPageNum {
        self.inner.vpn_range.get_start()
    }
    pub fn get_end<T: PageTable>(&self) -> VirtPageNum {
        self.inner.vpn_range.get_end()
    }

    /// Map vpns in `self` to the same ppns in `kernel_area` from `start_vpn_in_kernel_area`,
    /// range is depend on `self.vpn_range`.
    /// `page_table` and `self` should belong to the same `memory_set`.
    /// `vpn_range` in `kernel_area` should be broader than (or at least equal to) `self`.
    pub fn map_from_kernel_area<T: PageTable>(
        &mut self,
        page_table: &mut T,
        start_vpn_in_kernel_area: VirtPageNum,
    ) -> Result<(), ()> {
        let mut kernel_space = KERNEL_SPACE.lock();
        let kernel_area = kernel_space
            .get_area_by_vpn_range(start_vpn_in_kernel_area)
            .unwrap();
        let mut src_vpn = start_vpn_in_kernel_area;
        for vpn in self.inner.vpn_range {
            if let Some(frame) = kernel_area.inner.get_in_memory(&src_vpn) {
                let ppn = frame.ppn;
                if !page_table.is_mapped(vpn) {
                    self.inner.alloc_in_memory(vpn, frame.clone());
                    page_table.map(vpn, ppn, self.map_perm);
                } else {
                    error!("[map_from_kernel_area] user vpn already mapped!");
                    return Err(());
                }
            } else {
                error!("[map_from_kernel_area] kernel vpn invalid!");
                return Err(());
            }
            src_vpn = (src_vpn.0 + 1).into();
        }
        Ok(())
    }
    /// Unmap all pages in `self` from `page_table` using unmap_one()
    pub fn unmap<T: PageTable>(&mut self, page_table: &mut T) -> Result<(), MemoryError> {
        let mut has_unmapped_page = false;
        for vpn in self.inner.vpn_range {
            // it's normal to get an `Error` because we are using lazy alloc strategy
            // we still need to unmap remaining pages of `self`, just throw this `Error` to caller
            if let Err(MemoryError::NotMapped) = self.unmap_one(page_table, vpn) {
                has_unmapped_page = true;
            }
        }
        if has_unmapped_page {
            Err(MemoryError::NotMapped)
        } else {
            Ok(())
        }
    }
    pub fn copy_on_write<T: PageTable>(
        &mut self,
        page_table: &mut T,
        vpn: VirtPageNum,
    ) -> Result<PhysPageNum, MemoryError> {
        let old_frame = self.inner.remove_in_memory(&vpn).unwrap();
        if Arc::strong_count(&old_frame) == 1 {
            // don't need to copy
            // push back old frame and set pte flags to allow write
            let old_ppn = old_frame.ppn;
            self.inner.alloc_in_memory(vpn, old_frame);
            page_table.set_pte_flags(vpn, self.map_perm).unwrap();
            // Starting from this, the write (page) fault will not be triggered in this space,
            // for the pte permission now contains Write.
            trace!("[copy_on_write] no copy occurred");
            Ok(old_ppn)
        } else {
            // do copy in this case
            let old_ppn = old_frame.ppn;
            page_table.unmap(vpn);
            // alloc new frame
            let new_frame = unsafe { frame_alloc_uninit().unwrap() };
            let new_ppn = new_frame.ppn;
            self.inner.alloc_in_memory(vpn, new_frame);
            page_table.map(vpn, new_ppn, self.map_perm);
            // copy data
            new_ppn
                .get_bytes_array()
                .copy_from_slice(old_ppn.get_bytes_array());
            trace!("[copy_on_write] copy occurred");
            Ok(new_ppn)
        }
    }
    /// If `new_end` is equal to the current end of area, do nothing and return `Ok(())`.
    pub fn expand_to<T: PageTable>(&mut self, new_end: VirtAddr) -> Result<(), ()> {
        let new_end_vpn: VirtPageNum = new_end.ceil();
        let old_end_vpn = self.inner.vpn_range.get_end();
        if new_end_vpn < old_end_vpn {
            warn!(
                "[expand_to] new_end_vpn: {:?} is lower than old_end_vpn: {:?}",
                new_end_vpn, old_end_vpn
            );
            return Err(());
        }
        // `set_end` must be done before calling `map_one`
        // because `map_one` will insert frames into `data_frames`
        // if we don't `set_end` in advance, this insertion is out of bound
        self.inner.set_end(new_end_vpn)?;
        Ok(())
    }
    /// If `new_end` is equal to the current end of area, do nothing and return `Ok(())`.
    pub fn shrink_to<T: PageTable>(
        &mut self,
        page_table: &mut T,
        new_end: VirtAddr,
    ) -> Result<(), ()> {
        let new_end_vpn: VirtPageNum = new_end.ceil();
        let old_end_vpn = self.inner.vpn_range.get_end();
        if new_end_vpn > old_end_vpn {
            warn!(
                "[expand_to] new_end_vpn: {:?} is higher than old_end_vpn: {:?}",
                new_end_vpn, old_end_vpn
            );
            return Err(());
        }
        let mut has_unmapped_page = false;
        for vpn in VPNRange::new(new_end_vpn, old_end_vpn) {
            if let Err(_) = self.unmap_one(page_table, vpn) {
                has_unmapped_page = true;
            }
        }
        // `set_end` must be done after calling `map_one`
        // for the similar reason with `expand_to`
        self.inner.set_end(new_end_vpn)?;
        if has_unmapped_page {
            warn!("[shrink_to] Some pages are already unmapped, is it caused by lazy alloc?");
            Err(())
        } else {
            Ok(())
        }
    }
    /// If `new_start` is equal to the current start of area, do nothing and return `Ok(())`.
    pub fn rshrink_to<T: PageTable>(
        &mut self,
        page_table: &mut T,
        new_start: VirtAddr,
    ) -> Result<(), ()> {
        let new_start_vpn: VirtPageNum = new_start.floor();
        let old_start_vpn = self.inner.vpn_range.get_start();
        if new_start_vpn < old_start_vpn {
            warn!(
                "[rshrink_to] new_start_vpn: {:?} is lower than old_start_vpn: {:?}",
                new_start_vpn, old_start_vpn
            );
            return Err(());
        }
        let mut has_unmapped_page = false;
        for vpn in VPNRange::new(old_start_vpn, new_start_vpn) {
            if let Err(_) = self.unmap_one(page_table, vpn) {
                has_unmapped_page = true;
            }
        }
        // `set_start` must be done after calling `map_one`
        // for the similar reason with `expand_to`
        self.inner.set_start(new_start_vpn)?;
        if has_unmapped_page {
            warn!("[rshrink_to] Some pages are already unmapped, is it caused by lazy alloc?");
            Err(())
        } else {
            Ok(())
        }
    }
    pub fn check_overlapping(
        &self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
    ) -> Option<(VirtPageNum, VirtPageNum)> {
        let area_start_vpn = self.inner.vpn_range.get_start();
        let area_end_vpn = self.inner.vpn_range.get_end();
        if start_vpn == area_end_vpn {
            warn!("[check_overlapping] Is this correct?");
        }
        if end_vpn < area_start_vpn || start_vpn > area_end_vpn {
            return None;
        } else {
            let start = if start_vpn > area_start_vpn {
                start_vpn
            } else {
                area_start_vpn
            };
            let end = if end_vpn < area_end_vpn {
                end_vpn
            } else {
                area_end_vpn
            };
            return Some((start, end));
        }
    }
    pub fn into_two(&mut self, cut: VirtPageNum) -> Result<Self, ()> {
        let second_file = if let Some(file) = &self.map_file {
            let new_file = file.deep_clone();
            new_file
                .lseek(
                    (file.get_offset() + VirtAddr::from(cut).0
                        - VirtAddr::from(self.inner.vpn_range.get_start()).0)
                        as isize,
                    SeekWhence::SEEK_SET,
                )
                .unwrap();
            Some(new_file)
        } else {
            None
        };
        let second_frames = self.inner.into_two(cut)?;
        Ok(MapArea {
            inner: second_frames,
            map_type: self.map_type,
            map_perm: self.map_perm,
            map_file: second_file,
        })
    }
    pub fn into_three(
        &mut self,
        first_cut: VirtPageNum,
        second_cut: VirtPageNum,
    ) -> Result<(Self, Self), ()> {
        if self.map_file.is_some() {
            warn!("[into_three] break apart file-back MapArea!");
            return Err(());
        }
        let (second_frames, third_frames) = self.inner.into_three(first_cut, second_cut)?;
        Ok((
            MapArea {
                inner: second_frames,
                map_type: self.map_type,
                map_perm: self.map_perm,
                map_file: None,
            },
            MapArea {
                inner: third_frames,
                map_type: self.map_type,
                map_perm: self.map_perm,
                map_file: None,
            },
        ))
    }
    #[cfg(feature = "oom_handler")]
    pub fn do_oom<T: PageTable>(&mut self, page_table: &mut T) -> usize {
        let start_vpn = self.inner.vpn_range.get_start();
        let compressed_before = self.inner.compressed;
        let swapped_before = self.inner.swapped;
        warn!("{:?}", self.inner.active);
        while let Some(idx) = self.inner.active.pop_front() {
            let frame = &mut self.inner.frames[idx as usize];
            // first, try to compress
            match frame.zip() {
                Ok(zram_id) => {
                    page_table.unmap(VirtPageNum::from(start_vpn.0 + idx as usize));
                    self.inner.compressed += 1;
                    trace!("[do_oom] compress frame: {:?}, zram_id: {}", frame, zram_id);
                    continue;
                }
                Err(MemoryError::SharedPage) => continue,
                Err(MemoryError::ZramIsFull) => {}
                _ => unreachable!(),
            }
            // zram is full, try to swap out
            match frame.swap_out() {
                Ok(swap_id) => {
                    page_table.unmap(VirtPageNum::from(start_vpn.0 + idx as usize));
                    self.inner.swapped += 1;
                    trace!("[do_oom] swap out frame: {:?}, swap_id: {}", frame, swap_id);
                    continue;
                }
                Err(MemoryError::SharedPage) => continue,
                _ => unreachable!(),
            }
        }
        self.inner.compressed + self.inner.swapped - compressed_before - swapped_before
    }
    #[cfg(feature = "oom_handler")]
    pub fn force_swap<T: PageTable>(&mut self, page_table: &mut T) -> usize {
        let start_vpn = self.inner.vpn_range.get_start();
        let swapped_before = self.inner.swapped;
        warn!("{:?}", self.inner.active);
        while let Some(idx) = self.inner.active.pop_front() {
            let frame = &mut self.inner.frames[idx as usize];
            match frame.force_swap_out() {
                Ok(swap_id) => {
                    page_table.unmap(VirtPageNum::from(start_vpn.0 + idx as usize));
                    self.inner.swapped += 1;
                    trace!(
                        "[force_swap] swap out frame: {:?}, swap_id: {}",
                        frame,
                        swap_id
                    );
                    continue;
                }
                _ => unreachable!(),
            }
        }
        self.inner.swapped - swapped_before
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
impl MapPermission {
    #[inline(always)]
    pub fn from_ph_flags(ph_flags: xmas_elf::program::Flags) -> Self {
        let mut map_perm = MapPermission::U;
        if ph_flags.is_read() {
            map_perm |= MapPermission::R;
        }
        if ph_flags.is_write() {
            map_perm |= MapPermission::W;
        }
        if ph_flags.is_execute() {
            map_perm |= MapPermission::X;
        }
        map_perm
    }
}

bitflags! {
    pub struct MapFlags: usize {
        const MAP_SHARED            =   0x01;
        const MAP_PRIVATE           =   0x02;
        const MAP_SHARED_VALIDATE   =   0x03;
        const MAP_TYPE              =   0x0f;
        const MAP_FIXED             =   0x10;
        const MAP_ANONYMOUS         =   0x20;
        const MAP_NORESERVE         =   0x4000;
        const MAP_GROWSDOWN         =   0x0100;
        const MAP_DENYWRITE         =   0x0800;
        const MAP_EXECUTABLE        =   0x1000;
        const MAP_LOCKED            =   0x2000;
        const MAP_POPULATE          =   0x8000;
        const MAP_NONBLOCK          =   0x10000;
        const MAP_STACK             =   0x20000;
        const MAP_HUGETLB           =   0x40000;
        const MAP_SYNC              =   0x80000;
        const MAP_FIXED_NOREPLACE   =   0x100000;
        const MAP_FILE              =   0;
    }
}
