use super::map_area::*;
use super::page_table::PageTable;
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::arch::TrapContext;
use crate::arch::{MMIO, TICKS_PER_SEC};
use crate::fs::SeekWhence;
use crate::syscall::errno::*;
use crate::task::{
    current_task, trap_cx_bottom_from_tid, ustack_bottom_from_tid, AuxvEntry, AuxvType, ELFInfo,
};
use crate::{config::*, should_map_trampoline};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
use log::{debug, error, info, trace, warn};
use spin::Mutex;
extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
    fn ssignaltrampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet<crate::mm::KernelPageTableImpl>>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

#[allow(unused)]
#[derive(Debug)]
pub enum MemoryError {
    BadAddress,
    AreaNotFound,
    AlreadyMapped,
    NotMapped,
    NoPermission,
    NotInMemory,
    NotCompressed,
    NotSwappedOut,
    AlreadyAllocated,
    SharedPage,
    ZramIsFull,
    SwapIsFull,
    BeyondEOF,
}

/// The memory "space" as in user space or kernel space
pub struct MemorySet<T: PageTable> {
    page_table: T,
    /// The mapped area.
    /// Segments are implemented using this mechanism. In other words, they may be considered a subset of MapArea.
    /// Yet, other purposes may exist in this struct, such as file mapping.
    areas: Vec<MapArea>,
}

impl<T: PageTable> MemorySet<T> {
    /// Create a new struct with no information at all.
    pub fn new_bare_kern() -> Self {
        Self {
            page_table: T::new_kern_space(),
            areas: Vec::with_capacity(16),
        }
    }
    /// Create a new struct with no information at all.
    pub fn new_bare() -> Self {
        Self {
            page_table: T::new(),
            areas: Vec::with_capacity(16),
        }
    }
    /// Getter to the token of current memory space, or "this" page table.
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// Insert an anonymous segment containing the space between `start_va.floor()` to `end_va.ceil()`
    /// The space is allocated and added to the current MemorySet.
    /// # Prerequisite
    /// Assuming no conflicts. In other words, the space is NOT checked for space validity or overlap.
    /// It is merely mapped, pushed into the current memory set.
    /// Since CoW is implemented, the space is NOT allocated until a page fault is triggered.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission, None),
            None,
        )
        .unwrap();
    }
    /// Insert an anonymous segment containing the space between `start_va.floor()` to `end_va.ceil()`
    /// The space is allocated and added to the current MemorySet.
    /// # Prerequisite
    /// Assuming no conflicts. In other words, the space is NOT checked for space validity or overlap.
    /// It is merely mapped, pushed into the current memory set.
    /// Since CoW is implemented, the space is NOT allocated until a page fault is triggered.
    pub fn insert_program_area(
        &mut self,
        start_va: VirtAddr,
        permission: MapPermission,
        frames: Vec<Frame>,
    ) -> Result<(), ()> {
        let map_area = MapArea::from_existing_frame(start_va, MapType::Framed, permission, frames);
        self.push_no_alloc(map_area)?;
        Ok(())
    }
    pub fn remove_area_with_start_vpn(
        &mut self,
        start_vpn: VirtPageNum,
    ) -> Result<(), MemoryError> {
        if let Some((idx, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.get_start::<T>() == start_vpn)
        {
            let result = area.unmap(&mut self.page_table);
            self.areas.remove(idx);
            result
        } else {
            Err(MemoryError::AreaNotFound)
        }
    }
    /// Push a not-yet-mapped map_area into current MemorySet and copy the data into it if any, allocating the needed memory for the map.
    fn push(
        &mut self,
        mut map_area: MapArea,
        data: Option<&[u8]>,
    ) -> Result<(), (MemoryError, VirtPageNum)> {
        match data {
            Some(data) => {
                let mut start = 0;
                let len = data.len();
                for vpn in map_area.inner.vpn_range {
                    let ppn = map_area.map_one(&mut self.page_table, vpn)?;
                    let end = start + PAGE_SIZE;
                    let src = &data[start..len.min(end)];
                    ppn.get_bytes_array()[..src.len()].copy_from_slice(src);
                    start = end;
                }
            }
            None => {
                for vpn in map_area.inner.vpn_range {
                    map_area.map_one(&mut self.page_table, vpn)?;
                }
            }
        }
        self.areas.push(map_area);
        Ok(())
    }
    /// other parts will be zeroed
    fn push_with_offset(
        &mut self,
        mut map_area: MapArea,
        offset: usize,
        data: &[u8],
    ) -> Result<(), (MemoryError, VirtPageNum)> {
        let len = data.len();
        let mut vpn_iter = map_area.inner.vpn_range.into_iter();
        if let Some(vpn) = vpn_iter.next() {
            // special treatment for first page
            let first_ppn = map_area.map_one(&mut self.page_table, vpn)?;
            let first_dst = first_ppn.get_bytes_array();
            first_dst[..offset].fill(0);
            let first_src = &data[..len.min(PAGE_SIZE - offset)];
            first_dst[offset..offset + first_src.len()].copy_from_slice(first_src);

            let mut start = PAGE_SIZE - offset;
            for vpn in vpn_iter {
                let ppn = map_area.map_one(&mut self.page_table, vpn)?;
                let dst = ppn.get_bytes_array();
                let end = start + PAGE_SIZE;
                if start < len {
                    if len >= end {
                        let src = &data[start..end];
                        dst[..src.len()].copy_from_slice(src);
                    } else {
                        let src = &data[start..len];
                        dst[..src.len()].copy_from_slice(src);
                        dst[src.len()..].fill(0);
                    }
                } else {
                    dst.fill(0);
                }
                start = end;
            }
        }
        self.areas.push(map_area);
        Ok(())
    }

    pub fn get_area_by_vpn_range(
        &mut self,
        start_vpn_in_kernel_area: VirtPageNum,
    ) -> Option<&MapArea> {
        self.areas.iter().rev().find(|area| {
            area.get_start::<T>() <= start_vpn_in_kernel_area
                && start_vpn_in_kernel_area < area.get_end::<T>()
        })
    }

    /// Push the map area into the memory set without copying or allocation.
    pub fn push_no_alloc(&mut self, map_area: MapArea) -> Result<(), ()> {
        for vpn in map_area.inner.vpn_range {
            let frame = map_area.inner.get_in_memory(&vpn).unwrap();
            if !self.page_table.is_mapped(vpn) {
                //if not mapped
                self.page_table
                    .map(vpn, frame.ppn.clone(), map_area.map_perm);
            } else {
                return Err(());
            }
        }
        self.areas.push(map_area);
        Ok(())
    }
    pub fn last_mmap_area_idx(&self) -> Option<usize> {
        for (idx, area) in self.areas.iter().enumerate().rev().skip(SKIP_NUM) {
            let start_vpn = area.get_start::<T>();
            if start_vpn >= VirtAddr::from(USR_MMAP_END).into() {
                continue;
            } else if start_vpn >= VirtAddr::from(USR_MMAP_BASE).into()
                && start_vpn < VirtAddr::from(USR_MMAP_END).into()
            {
                return Some(idx);
            } else {
                return None;
            }
        }
        unreachable!();
    }
    pub fn highest_addr(&self) -> VirtAddr {
        self.areas.last().unwrap().get_end::<T>().into()
    }
    pub fn contains_valid_buffer(&self, buf: usize, size: usize, perm: MapPermission) -> bool {
        let start_vpn = VirtAddr::from(buf).floor();
        let end_vpn = VirtAddr::from(buf + size).ceil();
        self.areas
            .iter()
            .find(|area| {
                // If there is such a page in user space, and the addr is in the vpn range
                area.map_perm.contains(perm | MapPermission::U)
                    && area.get_start::<T>() <= start_vpn
                    && end_vpn <= area.get_end::<T>()
            })
            .is_some()
    }
    /// The REAL handler to page fault.
    /// Handles all types of page fault:(In regex:) "(Store|Load|Instruction)(Page)?Fault"
    /// Checks the permission to decide whether to copy.
    pub fn do_page_fault(&mut self, addr: VirtAddr) -> Result<PhysAddr, MemoryError> {
        let vpn = addr.floor();
        if let Some(area) = self.areas.iter_mut().find(|area| {
            area.map_perm.contains(MapPermission::R | MapPermission::U)// If there is such a page in user space
                && area.get_start::<T>() <= vpn// ...and the addr is in the vpn range
                && vpn < area.get_end::<T>()
        }) {
            if !self.page_table.is_mapped(vpn) {
                // lazy alloc file-backed page
                if let Some(file) = area.map_file.clone() {
                    let old_offset = file.get_offset();
                    let page_start_va = VirtAddr::from(vpn).0;
                    let area_start_va = VirtAddr::from(area.get_start::<T>()).0;
                    let offset_in_area = page_start_va - area_start_va;
                    // if offset exceed EOF, SIGBUS should be sent
                    if old_offset + offset_in_area > (file.get_size() + PAGE_SIZE - 1) & !0xfff {
                        return Err(MemoryError::BeyondEOF);
                    }
                    if area.map_perm.contains(MapPermission::W) {
                        let allocated_ppn = area.map_one_unchecked(&mut self.page_table, vpn);
                        file.lseek(offset_in_area as isize, SeekWhence::SEEK_CUR)
                            .unwrap();
                        file.read(None, unsafe {
                            core::slice::from_raw_parts_mut(
                                PhysAddr::from(allocated_ppn).0 as *mut u8,
                                PAGE_SIZE,
                            )
                        });
                        file.lseek(old_offset as isize, SeekWhence::SEEK_SET)
                            .unwrap();
                        Ok(allocated_ppn.offset(addr.page_offset()))
                    // map to phys page directly
                    } else {
                        let cache_phys_page = file
                            .get_single_cache(old_offset + offset_in_area)
                            .unwrap()
                            .try_lock()
                            .unwrap()
                            .get_tracker();
                        let cache_ppn = cache_phys_page.ppn;
                        self.page_table.map(vpn, cache_ppn, area.map_perm);
                        area.inner.alloc_in_memory(vpn, cache_phys_page);
                        Ok(cache_ppn.offset(addr.page_offset()))
                    }
                } else {
                    let frame = area.inner.get_mut(&vpn);
                    let allocated_ppn = match frame {
                        // Page table is not mapped, but frame is in memory.
                        Frame::InMemory(_) => unreachable!(),
                        Frame::Unallocated => {
                            info!("[do_page_fault] addr: {:?}, solution: lazy alloc", addr);
                            area.map_one_zeroed_unchecked(&mut self.page_table, vpn)
                        }
                        #[cfg(feature = "oom_handler")]
                        Frame::Compressed(_) => {
                            let ppn = frame.unzip().unwrap();
                            self.page_table.map(vpn, ppn, area.map_perm);
                            area.inner
                                .active
                                .push_back((vpn.0 - area.get_start::<T>().0) as u16);
                            area.inner.compressed -= 1;
                            info!("[do_page_fault] addr: {:?}, solution: decompress", addr);
                            ppn
                        }
                        #[cfg(feature = "oom_handler")]
                        Frame::SwappedOut(_) => {
                            let ppn = frame.swap_in().unwrap();
                            self.page_table.map(vpn, ppn, area.map_perm);
                            area.inner
                                .active
                                .push_back((vpn.0 - area.get_start::<T>().0) as u16);
                            area.inner.swapped -= 1;
                            info!("[do_page_fault] addr: {:?}, solution: swap in", addr);
                            ppn
                        }
                    };
                    Ok(allocated_ppn.offset(addr.page_offset()))
                }
            } else {
                // mapped before the assignment
                if area.map_perm.contains(MapPermission::W) {
                    // Whoever triggers this fault shall cause the area to be copied into a new area.
                    let allocated_ppn = area.copy_on_write(&mut self.page_table, vpn)?;
                    info!("[do_page_fault] addr: {:?}, solution: copy on write", addr);
                    Ok(allocated_ppn.offset(addr.page_offset()))
                } else {
                    // Write without permission
                    error!(
                        "[do_page_fault] addr: {:?}, result: write no permission",
                        addr
                    );
                    Err(MemoryError::NoPermission)
                }
            }
        } else {
            // In all segments, nothing matches the requirements. Throws.
            error!("[do_page_fault] addr: {:?}, result: bad addr", addr);
            Err(MemoryError::BadAddress)
        }
    }
    #[cfg(feature = "oom_handler")]
    pub fn do_shallow_clean(&mut self) -> usize {
        let page_table = &mut self.page_table;
        self.areas
            .iter_mut()
            .filter(|area| {
                let start_vpn = area.get_start::<T>();
                start_vpn.0 >= (USR_MMAP_BASE >> PAGE_SIZE_BITS)
                    && start_vpn.0 < (TASK_SIZE >> PAGE_SIZE_BITS)
                    && area.map_file.is_none()
            })
            .map(|area| area.do_oom(page_table))
            .sum()
    }
    #[cfg(feature = "oom_handler")]
    pub fn do_deep_clean(&mut self) -> usize {
        let page_table = &mut self.page_table;
        self.areas
            .iter_mut()
            .filter(|area| {
                area.get_start::<T>().0 < (TASK_SIZE >> PAGE_SIZE_BITS) && area.map_file.is_none()
            })
            .map(|area| {
                if area.get_start::<T>().0 < USR_MMAP_BASE >> PAGE_SIZE_BITS {
                    area.force_swap(page_table)
                } else {
                    area.do_oom(page_table)
                }
            })
            .sum()
    }
    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            MapPermission::R | MapPermission::X,
        );
    }
    /// Can be accessed in user mode.
    fn map_signaltrampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(SIGNAL_TRAMPOLINE).into(),
            PhysAddr::from(ssignaltrampoline as usize).into(),
            MapPermission::R | MapPermission::X | MapPermission::U,
        );
    }
    /// Create an empty kernel space.
    /// Without kernel stacks. (Is it done with .bss?)
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare_kern();
        // map trampoline
        if should_map_trampoline!() {
            memory_set.map_trampoline();
        }
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        macro_rules! anonymous_identical_map {
            ($begin:expr,$end:expr,$permission:expr) => {
                memory_set
                    .push(
                        MapArea::new(
                            ($begin as usize).into(),
                            ($end as usize).into(),
                            MapType::Identical,
                            $permission,
                            None,
                        ),
                        None,
                    )
                    .unwrap();
            };
            ($name:literal,$begin:expr,$end:expr,$permission:expr) => {
                println!("mapping {}", $name);
                anonymous_identical_map!($begin, $end, $permission);
            };
        }
        anonymous_identical_map!(
            ".text section",
            stext,
            etext,
            MapPermission::R | MapPermission::X
        );
        anonymous_identical_map!(".rodata section", srodata, erodata, MapPermission::R); // read only section
        anonymous_identical_map!(
            ".data section",
            sdata,
            edata,
            MapPermission::R | MapPermission::W
        );
        anonymous_identical_map!(
            ".bss section",
            sbss_with_stack,
            ebss,
            MapPermission::R | MapPermission::W
        );
        anonymous_identical_map!(
            "physical memory",
            ekernel,
            MEMORY_END,
            MapPermission::R | MapPermission::W
        );

        println!("mapping memory-mapped registers");
        for pair in MMIO {
            anonymous_identical_map!(
                (*pair).0,
                ((*pair).0 + (*pair).1),
                MapPermission::R | MapPermission::W
            );
        }
        memory_set
    }
    pub fn map_elf(&mut self, elf: &xmas_elf::ElfFile) -> Result<(usize, ELFInfo), isize> {
        let bias = match elf.header.pt2.type_().as_type() {
            // static
            xmas_elf::header::Type::Executable => 0,
            xmas_elf::header::Type::SharedObject => {
                match elf
                    .program_iter()
                    .filter(|ph| ph.get_type().unwrap() == xmas_elf::program::Type::Interp)
                    .count()
                {
                    // It's a loader!
                    0 => ELF_DYN_BASE,
                    // It's a dynamically linked ELF.
                    1 => 0,
                    // Emmm, It has multiple interpreters.
                    _ => return Err(EINVAL),
                }
            }
            _ => return Err(ENOEXEC),
        };

        let mut program_break: Option<usize> = None;
        let mut interp_entry: Option<usize> = None;
        let mut interp_base: Option<usize> = None;
        let mut load_addr: Option<usize> = None; // top va of ELF which points to ELF header

        for ph in elf.program_iter() {
            // Map only when the sections that is to be loaded.
            match ph.get_type().unwrap() {
                xmas_elf::program::Type::Load => {
                    let start_va: VirtAddr = (ph.virtual_addr() as usize + bias).into();
                    let end_va: VirtAddr =
                        ((ph.virtual_addr() + ph.mem_size()) as usize + bias).into();
                    let start_va_page_offset = start_va.page_offset();

                    let map_perm = MapPermission::from_ph_flags(ph.flags());
                    if load_addr.is_none() {
                        load_addr = Some(start_va.into());
                    }
                    let mut map_area =
                        MapArea::new(start_va, end_va, MapType::Framed, map_perm, None);
                    // Virtual addr is 4K-aligned
                    if (start_va_page_offset & (PAGE_SIZE - 1)) == 0
                    // Physical addr is 4K-aligned
                        && (ph.offset() as usize & (PAGE_SIZE - 1)) == 0
                        && ph.file_size() != 0
                        && !map_perm.contains(MapPermission::W)
                    {
                        // Size in virtual addr is equal to size in physical addr
                        assert_eq!(
                            VirtAddr::from(ph.file_size() as usize).ceil().0,
                            map_area.get_end::<T>().0 - map_area.get_start::<T>().0
                        );

                        let kernel_start_vpn =
                            (VirtAddr::from(elf.input.as_ptr() as usize + (ph.offset() as usize)))
                                .floor();
                        map_area
                            .map_from_kernel_area(&mut self.page_table, kernel_start_vpn)
                            .unwrap();
                        self.areas.push(map_area);
                    } else {
                        if let Err(_) = self.push_with_offset(
                            map_area,
                            start_va_page_offset,
                            &elf.input
                                [ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                        ) {
                            panic!("[map_elf] Target addr already mapped.")
                        };
                    }
                    program_break = Some(VirtAddr::from(end_va.ceil()).0);
                    trace!(
                        "[map_elf] start_va = 0x{:X}; end_va = 0x{:X}, offset = 0x{:X}",
                        start_va.0,
                        end_va.0,
                        start_va_page_offset
                    );
                }
                xmas_elf::program::Type::Interp => {
                    assert!(elf.input[(ph.offset() + ph.file_size()) as usize] == b'\0');
                    let path = String::from_utf8_lossy(
                        &elf.input
                            [ph.offset() as usize..(ph.offset() + ph.file_size() - 1) as usize],
                    );
                    debug!("[map_elf] Found interpreter path: {}", path);
                    let interp_data = crate::task::load_elf_interp(&path)?;
                    let interp = xmas_elf::ElfFile::new(interp_data).unwrap();
                    let (_, interp_info) = self.map_elf(&interp)?;
                    interp_entry = Some(interp_info.entry);
                    interp_base = Some(interp_info.base);
                    KERNEL_SPACE
                        .lock()
                        .remove_area_with_start_vpn(
                            VirtAddr::from(interp_data.as_ptr() as usize).ceil(),
                        )
                        .unwrap();
                }
                _ => {}
            }
        }
        match (program_break, load_addr) {
            (Some(program_break), Some(load_addr)) => Ok((
                program_break,
                ELFInfo {
                    entry: elf.header.pt2.entry_point() as usize + bias,
                    interp_entry,
                    base: if let Some(interp_base) = interp_base {
                        interp_base
                    } else {
                        bias
                    },
                    phnum: elf.header.pt2.ph_count() as usize,
                    phent: elf.header.pt2.ph_entry_size() as usize,
                    phdr: load_addr + elf.header.pt2.ph_offset() as usize,
                },
            )),
            _ => Err(ENOEXEC),
        }
    }
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> Result<(Self, usize, ELFInfo), isize> {
        let mut memory_set = Self::new_bare();
        // map trampoline
        if should_map_trampoline!() {
            memory_set.map_trampoline();
        }
        // map signaltrampoline
        memory_set.map_signaltrampoline();
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let (program_break, elf_info) = memory_set.map_elf(&elf)?;

        Ok((memory_set, program_break, elf_info))
    }
    pub fn from_existing_user(user_space: &mut MemorySet<T>) -> MemorySet<T> {
        let mut memory_set = Self::new_bare();
        // map trampoline
        if should_map_trampoline!() {
            memory_set.map_trampoline();
        }
        // map signaltrampoline
        memory_set.map_signaltrampoline();
        // map data sections/user heap/mmap area/user stack
        for i in 0..user_space.areas.len() - 1 {
            // user_space.areas[i]
            let mut new_area = user_space.areas[i].clone();
            new_area
                .map_from_existing_page_table(
                    &mut memory_set.page_table,
                    &mut user_space.page_table,
                )
                .unwrap();
            memory_set.areas.push(new_area);
            debug!(
                "[fork] map shared area: {:?}",
                user_space.areas[i].inner.vpn_range
            );
        }
        // copy trap context area
        let trap_cx_area = user_space.areas.last().unwrap();
        let area = MapArea::from_another(trap_cx_area);
        let vpn = trap_cx_area.get_start::<T>();
        memory_set
            .push(
                area,
                Some(
                    user_space
                        .translate(vpn)
                        .unwrap()
                        .start_addr()
                        .get_bytes_ref::<TrapContext>(),
                ),
            )
            .unwrap();

        debug!(
            "[fork] copy trap_cx area: {:?}",
            trap_cx_area.inner.vpn_range
        );
        memory_set
    }
    pub fn activate(&self) {
        self.page_table.activate()
    }
    /// Translate the `vpn` into its corresponding `Some(PageTableEntry)` in the current memory set if exists
    /// `None` is returned if nothing is found.
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.page_table.translate(vpn)
    }
    #[allow(unused)]
    pub fn set_pte_flags(&mut self, vpn: VirtPageNum, flags: MapPermission) -> Result<(), ()> {
        self.page_table.set_pte_flags(vpn, flags)
    }
    #[allow(unused)]
    pub fn clear_access_bit(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        self.page_table.clear_access_bit(vpn)
    }
    #[allow(unused)]
    pub fn clear_dirty_bit(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        self.page_table.clear_dirty_bit(vpn)
    }
    pub fn recycle_data_pages(&mut self) {
        //*self = Self::new_bare();
        self.areas.clear();
    }
    #[allow(unused)]
    // debug use only
    pub fn show_areas(&self) {
        self.areas.iter().for_each(|area| {
            let start_vpn = area.get_start::<T>();
            let end_vpn = area.get_end::<T>();
            error!(
                "[show_areas] start_vpn: {:?}, end_vpn: {:?}, map_perm: {:?}",
                start_vpn, end_vpn, area.map_perm
            );
        })
    }
    pub fn sbrk(&mut self, heap_pt: usize, heap_bottom: usize, increment: isize) -> usize {
        let old_pt: usize = heap_pt;
        let new_pt: usize = old_pt + increment as usize;
        if increment > 0 {
            let limit = heap_bottom + USER_HEAP_SIZE;
            if new_pt > limit {
                warn!(
                    "[sbrk] out of the upperbound! upperbound: {:X}, old_pt: {:X}, new_pt: {:X}",
                    limit, old_pt, new_pt
                );
                return old_pt;
            } else {
                self.mmap(
                    old_pt,
                    increment as usize,
                    MapPermission::R | MapPermission::W | MapPermission::U,
                    MapFlags::MAP_ANONYMOUS | MapFlags::MAP_FIXED | MapFlags::MAP_PRIVATE,
                    1usize.wrapping_neg(),
                    0,
                );
                trace!("[sbrk] heap area expanded to {:X}", new_pt);
            }
        } else if increment < 0 {
            // shrink to `heap_bottom` would cause duplicated insertion of heap area in future
            // so we simply reject it here
            if new_pt <= heap_bottom {
                warn!(
                    "[sbrk] out of the lowerbound! lowerbound: {:X}, old_pt: {:X}, new_pt: {:X}",
                    heap_bottom, old_pt, new_pt
                );
                return old_pt;
            // attention that if the process never call sbrk before, it would have no heap area
            // we only do shrinking when it does have a heap area
            } else {
                self.munmap(old_pt, increment as usize).unwrap();
                trace!("[sbrk] heap area shrinked to {:X}", new_pt);
            }
            // we need to adjust `heap_pt` if it's not out of bound
            // in spite of whether the process has a heap area
        }
        new_pt
    }
    pub fn mmap(
        &mut self,
        start: usize,
        len: usize,
        prot: MapPermission,
        flags: MapFlags,
        fd: usize,
        offset: usize,
    ) -> isize {
        // not aligned on a page boundary
        if start & 0xfff != 0 {
            return EINVAL;
        }
        let len = if len == 0 { PAGE_SIZE } else { len };
        let task = current_task().unwrap();
        let idx = self.last_mmap_area_idx();
        let start_va: VirtAddr = if flags.contains(MapFlags::MAP_FIXED) {
            // unmap if exists
            unsafe { self.munmap(start, len).unwrap_unchecked() };
            start.into()
        } else {
            if let Some(idx) = idx {
                let area = &mut self.areas[idx];
                if flags.contains(MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS)
                    && prot == area.map_perm
                    && area.map_file.is_none()
                {
                    debug!("[mmap] merge with previous area, call expand_to");
                    let end_va: VirtAddr = area.get_end::<T>().into();
                    area.expand_to::<T>(VirtAddr::from(end_va.0 + len)).unwrap();
                    return end_va.0 as isize;
                }
                area.get_end::<T>().into()
            } else {
                USR_MMAP_BASE.into()
            }
        };
        let mut new_area = MapArea::new(
            start_va,
            VirtAddr::from(start_va.0 + len),
            MapType::Framed,
            prot,
            None,
        );
        if !flags.contains(MapFlags::MAP_ANONYMOUS) {
            warn!("[mmap] file-backed map!");
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => {
                    if !file_descriptor.readable() {
                        return EACCES;
                    }
                    let file = file_descriptor.file.deep_clone();
                    file.lseek(offset as isize, SeekWhence::SEEK_SET).unwrap();
                    new_area.map_file = Some(file);
                }
                Err(errno) => return errno,
            }
        }
        // insert MapArea and keep the order
        if let Some((idx, _)) = self
            .areas
            .iter()
            .enumerate()
            .skip_while(|(_, area)| area.get_start::<T>() >= VirtAddr::from(USR_MMAP_END).into())
            .find(|(_, area)| area.get_start::<T>() >= start_va.into())
        {
            self.areas.insert(idx, new_area);
        } else {
            error!("[MemorySet::mmap] No area found higher than new_area {:?} in beginning address. TRAMPOLINES may have been mapped to wrong places!",new_area);
            self.areas.push(new_area);
        }
        start_va.0 as isize
    }
    pub fn munmap(&mut self, start: usize, len: usize) -> Result<(), isize> {
        let start_va = VirtAddr::from(start);
        let end_va = VirtAddr::from(start + len);
        if !start_va.aligned() {
            warn!("[munmap] Not aligned");
            return Err(EINVAL);
        }
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let page_table = &mut self.page_table;
        let mut found_area = false;
        let mut delete: Vec<usize> = Vec::new();
        let mut break_apart_idx: Option<usize> = None;
        self.areas.iter_mut().enumerate().for_each(|(idx, area)| {
            if let Some((overlap_start, overlap_end)) = area.check_overlapping(start_vpn, end_vpn) {
                found_area = true;
                let area_start_vpn: VirtPageNum = area.get_start::<T>();
                let area_end_vpn = area.get_end::<T>();
                if overlap_start == area_start_vpn && overlap_end == area_end_vpn {
                    trace!("[munmap] unmap whole area, idx: {}", idx);
                    if let Err(_) = area.unmap(page_table) {
                        warn!(
                            "[munmap] Some pages are already unmapped, is it caused by lazy alloc?"
                        );
                    }
                    delete.push(idx);
                } else if overlap_start == area_start_vpn {
                    trace!("[munmap] unmap lower part, call rshrink_to");
                    if let Err(_) = area.rshrink_to(page_table, VirtAddr::from(overlap_end)) {
                        warn!(
                            "[munmap] Some pages are already unmapped, is it caused by lazy alloc?"
                        );
                    }
                } else if overlap_end == area_end_vpn {
                    trace!("[munmap] unmap higher part, call shrink_to");
                    if let Err(_) = area.shrink_to(page_table, VirtAddr::from(overlap_start)) {
                        warn!(
                            "[munmap] Some pages are already unmapped, is it caused by lazy alloc?"
                        );
                    }
                } else {
                    trace!("[munmap] unmap internal part, call into_three");
                    break_apart_idx = Some(idx);
                }
            }
        });
        for idx in delete.into_iter().rev() {
            self.areas.remove(idx);
        }
        if let Some(idx) = break_apart_idx {
            let (mut second, third) = self.areas[idx].into_three(start_vpn, end_vpn).unwrap();
            if let Err(_) = second.unmap(page_table) {
                warn!("[munmap] Some pages are already unmapped, is it caused by lazy alloc?");
            }
            self.areas.insert(idx + 1, third);
        }
        if found_area {
            Ok(())
        } else {
            Err(EINVAL)
        }
    }
    pub fn mprotect(&mut self, addr: usize, len: usize, prot: usize) -> Result<(), isize> {
        let start_va = VirtAddr::from(addr);
        let end_va = VirtAddr::from(addr + len);
        // addr is not a multiple of the system page size.
        if !start_va.aligned() {
            warn!("[mprotect] Not aligned");
            return Err(EINVAL);
        }
        // here (prot << 1) is identical to BitFlags of X/W/R in pte flags
        let prot = MapPermission::from_bits(((prot as u8) << 1) | (1 << 4)).unwrap();
        warn!(
            "[mprotect] addr: {:X}, len: {:X}, prot: {:?}",
            addr, len, prot
        );
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let result = self.areas.iter().enumerate().find(|(_, area)| {
            area.get_start::<T>() <= start_vpn && start_vpn < area.get_end::<T>()
        });
        match result {
            Some((idx, _)) => {
                let area_start_vpn = self.areas[idx].get_start::<T>();
                let area_end_vpn = self.areas[idx].get_end::<T>();
                // Addresses in the range [addr, addr+len-1] are invalid for the address space of the process,
                // or specify one or more pages that are not mapped.
                if end_vpn > area_end_vpn {
                    warn!("[mprotect] addr: {:X} is not in any MapArea", addr);
                    return Err(ENOMEM);
                }
                let area: &mut MapArea = if start_vpn == area_start_vpn && end_vpn == area_end_vpn {
                    trace!("[mprotect] change prot of whole area, idx: {}", idx);
                    &mut self.areas[idx]
                } else if start_vpn == area_start_vpn {
                    trace!("[mprotect] change prot of lower part");
                    let second = self.areas[idx].into_two(end_vpn).unwrap();
                    self.areas.insert(idx + 1, second);
                    // important, keep the order of areas
                    &mut self.areas[idx]
                } else if end_vpn == area_end_vpn {
                    trace!("[mprotect] change prot of higher part");
                    let second = self.areas[idx].into_two(start_vpn).unwrap();
                    self.areas.insert(idx + 1, second);
                    &mut self.areas[idx + 1]
                } else {
                    trace!("[mprotect] change prot of internal part, call into_three");
                    let (second, third) = self.areas[idx].into_three(start_vpn, end_vpn).unwrap();
                    self.areas.insert(idx + 1, second);
                    self.areas.insert(idx + 2, third);
                    &mut self.areas[idx + 1]
                };
                let page_table = &mut self.page_table;
                let mut has_unmapped_page = false;
                for vpn in area.inner.vpn_range {
                    // Clear W prot, or CoW pages may be written unexpectedly.
                    // And those pages will gain W prot by CoW.
                    if let Err(_) = page_table.set_pte_flags(vpn, prot - MapPermission::W) {
                        has_unmapped_page = true;
                    }
                }
                if has_unmapped_page {
                    warn!("[mprotect] Some pages are not mapped, is it caused by lazy alloc?");
                }
                // If `prot` contains W, store page fault & CoW will occur.
                area.map_perm = prot;
            }
            None => {
                warn!("[mprotect] addr is not a valid pointer");
                return Err(EINVAL);
            }
        }
        Ok(())
    }
    pub fn create_elf_tables(
        &self,
        mut user_sp: usize,
        argv_vec: &Vec<String>,
        envp_vec: &Vec<String>,
        elf_info: &ELFInfo,
    ) -> usize {
        // go down to the stack page (important!) and align
        user_sp -= 2 * core::mem::size_of::<usize>();
        // because size of parameters is almost never more than PAGE_SIZE,
        // so I decide to use physical address directly for better performance
        let mut phys_user_sp = T::from_token(self.token())
            .translate_va(VirtAddr::from(user_sp))
            .unwrap()
            .0;
        let virt_phys_offset = user_sp - phys_user_sp;
        let phys_start = phys_user_sp;
        // unsafe code is efficient code! here we go!
        fn copy_to_user_string_unchecked(src: &str, dst: *mut u8) {
            let size = src.len();
            unsafe {
                core::slice::from_raw_parts_mut(dst, size)
                    .copy_from_slice(core::slice::from_raw_parts(src.as_ptr(), size));
                // adapt to C-style string
                *dst.add(size) = b'\0';
            }
        }

        // we don't care about the order of env...
        let mut envp_user = Vec::<*const u8>::new();
        for env in envp_vec.iter() {
            phys_user_sp -= env.len() + 1;
            envp_user.push((phys_user_sp + virt_phys_offset) as *const u8);
            copy_to_user_string_unchecked(env, phys_user_sp as *mut u8);
        }
        envp_user.push(core::ptr::null());

        // we don't care about the order of arg, too...
        let mut argv_user = Vec::<*const u8>::new();
        for arg in argv_vec.iter() {
            phys_user_sp -= arg.len() + 1;
            argv_user.push((phys_user_sp + virt_phys_offset) as *const u8);
            copy_to_user_string_unchecked(arg, phys_user_sp as *mut u8);
        }
        argv_user.push(core::ptr::null());
        // align downward to usize (64bit)
        phys_user_sp &= !0x7;

        // 16 random bytes
        phys_user_sp -= 2 * core::mem::size_of::<usize>();
        // should be virt addr!
        let random_bits_ptr = phys_user_sp + virt_phys_offset;
        unsafe {
            *(phys_user_sp as *mut usize) = 0xdeadbeefcafebabe;
            *(phys_user_sp as *mut usize).add(1) = 0xdeadbeefcafebabe;
        }
        // padding
        phys_user_sp -= core::mem::size_of::<usize>();
        unsafe {
            *(phys_user_sp as *mut usize) = 0x0000000000000000;
        }
        let auxv = [
            // AuxvEntry::new(AuxvType::SYSINFO_EHDR, vDSO_mapping);
            // AuxvEntry::new(AuxvType::L1I_CACHESIZE, 0);
            // AuxvEntry::new(AuxvType::L1I_CACHEGEOMETRY, 0);
            // AuxvEntry::new(AuxvType::L1D_CACHESIZE, 0);
            // AuxvEntry::new(AuxvType::L1D_CACHEGEOMETRY, 0);
            // AuxvEntry::new(AuxvType::L2_CACHESIZE, 0);
            // AuxvEntry::new(AuxvType::L2_CACHEGEOMETRY, 0);
            // `0x112d` means IMADZifenciC, aka gc
            AuxvEntry::new(AuxvType::HWCAP, 0x112d),
            AuxvEntry::new(AuxvType::PAGESZ, PAGE_SIZE),
            AuxvEntry::new(AuxvType::CLKTCK, TICKS_PER_SEC),
            AuxvEntry::new(AuxvType::PHDR, elf_info.phdr),
            AuxvEntry::new(AuxvType::PHENT, elf_info.phent),
            AuxvEntry::new(AuxvType::PHNUM, elf_info.phnum),
            AuxvEntry::new(AuxvType::BASE, elf_info.base),
            AuxvEntry::new(AuxvType::FLAGS, 0),
            AuxvEntry::new(AuxvType::ENTRY, elf_info.entry),
            AuxvEntry::new(AuxvType::UID, 0),
            AuxvEntry::new(AuxvType::EUID, 0),
            AuxvEntry::new(AuxvType::GID, 0),
            AuxvEntry::new(AuxvType::EGID, 0),
            AuxvEntry::new(AuxvType::SECURE, 0),
            AuxvEntry::new(AuxvType::RANDOM, random_bits_ptr as usize),
            AuxvEntry::new(
                AuxvType::EXECFN,
                argv_user.first().copied().unwrap() as usize,
            ),
            AuxvEntry::new(AuxvType::NULL, 0),
        ];
        phys_user_sp -= auxv.len() * core::mem::size_of::<AuxvEntry>();
        unsafe {
            core::slice::from_raw_parts_mut(phys_user_sp as *mut AuxvEntry, auxv.len())
                .copy_from_slice(auxv.as_slice());
        }
        phys_user_sp -= envp_user.len() * core::mem::size_of::<usize>();
        unsafe {
            core::slice::from_raw_parts_mut(phys_user_sp as *mut *const u8, envp_user.len())
                .copy_from_slice(envp_user.as_slice());
        }
        phys_user_sp -= argv_user.len() * core::mem::size_of::<usize>();
        unsafe {
            core::slice::from_raw_parts_mut(phys_user_sp as *mut *const u8, argv_user.len())
                .copy_from_slice(argv_user.as_slice());
        }
        phys_user_sp -= core::mem::size_of::<usize>();
        unsafe {
            *(phys_user_sp as *mut usize) = argv_vec.len();
        }

        user_sp = phys_user_sp + virt_phys_offset;

        // unlikely, if `start` and `end` are in different pages, we should panic
        assert_eq!(phys_start & !0xfff, phys_user_sp & !0xfff);

        // print user stack
        // let mut phys_addr = phys_user_sp & !0xf;
        // while phys_start >= phys_addr {
        //     trace!(
        //         "0x{:0>16X}:    {:0>16X}  {:0>16X}",
        //         phys_addr + virt_phys_offset,
        //         unsafe { *(phys_addr as *mut usize) },
        //         unsafe { *((phys_addr + core::mem::size_of::<usize>()) as *mut usize) }
        //     );
        //     phys_addr += 2 * core::mem::size_of::<usize>();
        // }
        user_sp
    }
    pub fn alloc_user_res(&mut self, tid: usize, alloc_stack: bool) {
        if alloc_stack {
            let ustack_bottom = ustack_bottom_from_tid(tid);
            let ustack_top = ustack_bottom - USER_STACK_SIZE;
            trace!(
                "[alloc_user_res] user stack start_va: {:X}, end_va: {:X}",
                ustack_top,
                ustack_bottom
            );
            // alloc user stack
            self.insert_framed_area(
                ustack_top.into(),
                ustack_bottom.into(),
                MapPermission::R | MapPermission::W | MapPermission::U,
            );
            trace!("[alloc_user_res] done");
        } else {
            debug!(
                "[alloc_user_res] user stack is not allocated (stack is designated in sys_clone)"
            );
        }
        // alloc trap_cx
        let trap_cx_bottom = trap_cx_bottom_from_tid(tid);
        let trap_cx_top = trap_cx_bottom + PAGE_SIZE;
        self.insert_framed_area(
            trap_cx_bottom.into(),
            trap_cx_top.into(),
            MapPermission::R | MapPermission::W,
        );
        trace!(
            "[alloc_user_res] trap context start_va: {:X}, end_va: {:X}",
            trap_cx_bottom,
            trap_cx_top
        );
    }

    pub fn dealloc_user_res(&mut self, tid: usize) {
        // dealloc ustack manually
        let ustack_top_va: VirtAddr = (ustack_bottom_from_tid(tid) - USER_STACK_SIZE).into();
        if let Err(err) = self.remove_area_with_start_vpn(ustack_top_va.into()) {
            match err {
                MemoryError::AreaNotFound => {
                    warn!("[dealloc_user_res] user stack is not allocated")
                }
                MemoryError::NotMapped => {
                    warn!(
                        "[dealloc_user_res] user stack is partially unmapped, is it caused by oom?"
                    )
                }
                _ => unreachable!(),
            }
        }
        // dealloc trap_cx manually
        let trap_cx_bottom_va: VirtAddr = trap_cx_bottom_from_tid(tid).into();
        self.remove_area_with_start_vpn(trap_cx_bottom_va.into())
            .unwrap();
    }

    pub fn is_dirty(&self, ppn: PhysPageNum) -> Option<bool> {
        self.page_table.is_dirty((ppn.0).into())
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space.page_table.writable(mid_text.floor()).unwrap(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .writable(mid_rodata.floor())
            .unwrap(),
        false,
    );
    assert_eq!(
        kernel_space
            .page_table
            .executable(mid_data.floor())
            .unwrap(),
        false,
    );
    info!("remap_test passed!");
}

pub fn check_page_fault(addr: VirtAddr) -> Result<PhysAddr, isize> {
    // This is where we handle the page fault.
    super::frame_reserve(3);
    let task = current_task().unwrap();
    match task.vm.lock().do_page_fault(addr) {
        Ok(pa) => return Ok(pa),
        Err(MemoryError::BeyondEOF)
        | Err(MemoryError::NoPermission)
        | Err(MemoryError::BadAddress) => {
            return Err(EFAULT);
        }
        _ => unreachable!(),
    };
}
