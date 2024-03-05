use super::layout::{FATDirEnt, FATShortDirEnt};
use super::vfs::InodeLock;
use super::Inode;
use alloc::string::String;
use spin::*;

/// `DirIterMode` describe `DirIter`'s iterate mode
/// + `Long`: Iterate to the next long directory entry
/// + `Short`: Iterate to the next short directory entry
/// + `Used`: Iterate to the next used(long or short) directory entry
/// + `Unused`: Iterate to the next unused(not long and not short) directory entry
/// + `Enum`: Iterate to the next arbitrary directory entry
#[allow(unused)]
pub enum DirIterMode {
    Long,
    Short,
    Used,
    Unused,
    Enum,
}
pub const FORWARD: bool = true;
pub const BACKWARD: bool = false;
const STEP_SIZE: u32 = core::mem::size_of::<FATDirEnt>() as u32;
/// `DirIter`: an iterator for directory file
/// This iterator has 5 modes(see `DirIterMode` for detail),
/// 2 direction(forward and backward)
/// # WARNING
/// If the offset is None and the direction is forward, its value will become Some(0) after the next iteration.
/// If the offset is Some(0) and the direction is backward, its value will become None after the next iteration.
/// For `Enum` mode, it's valid to iterate to `last_and_unused` entry, but for other this operation is invalid.
/// Otherwise, if the offset is at the boundary (the next iteration will be invalid), its value will not change after the next iteration.
pub struct DirIter<'a, 'b> {
    /// The lock of directory file's `file content`
    pub inode_lock: &'a RwLockWriteGuard<'b, InodeLock>,
    /// The current offset in file
    offset: Option<u32>,
    mode: DirIterMode,
    direction: bool,
    /// The pointer of inode
    inode: &'a Inode,
}

impl<'a, 'b> DirIter<'a, 'b> {
    /// Constructor for `DirIter`
    /// # Arguments    
    /// + `lock`: The lock of target file content
    /// + `offset`: The start of offset
    /// + `mode`: The iterative mode
    /// + `direction`: The iterative direction
    /// + `inode`: The pointer of target file inode
    /// # Return Value
    /// An `DirIter`
    pub fn new(
        inode_lock: &'a RwLockWriteGuard<'b, InodeLock>,
        offset: Option<u32>,
        mode: DirIterMode,
        direction: bool,
        inode: &'a Inode,
    ) -> Self {
        Self {
            inode_lock,
            offset,
            mode,
            direction,
            inode,
        }
    }
    #[inline(always)]
    /// Get iterator corresponding offset
    pub fn get_offset(&self) -> Option<u32> {
        self.offset
    }
    #[inline(always)]
    /// Sets the offset to make the first iteration of the iterator to the target `offset`
    /// # Arguments
    /// + `offset`: The target offset we want after the first iteration
    pub fn set_iter_offset(&mut self, offset: u32) {
        if self.direction {
            if offset == 0 {
                self.offset = None;
            } else {
                self.offset = Some(offset - STEP_SIZE);
            }
        } else {
            self.offset = Some(offset + STEP_SIZE);
        }
    }
    /// Get `FATDirEnt` content corresponding to `offset`.
    /// # Return Value
    /// If successful, it will return a `FATDirEnt`
    /// Otherwise, it will return None
    pub fn current_clone(&mut self) -> Option<FATDirEnt> {
        let mut dir_ent = FATDirEnt::empty();
        if self.offset.is_some()
            && self.offset.unwrap() < self.inode.get_file_size_wlock(self.inode_lock)
            && self.inode.read_at_block_cache_wlock(
                &self.inode_lock,
                self.offset.unwrap() as usize,
                dir_ent.as_bytes_mut(),
            ) != 0
        {
            Some(dir_ent)
        } else {
            None
        }
    }
    /// Write `ent` to the directory entry corresponding to iterator.
    /// # Arguments
    /// + `ent`: The directory entry we want to write to
    /// # Warning
    /// If write failed, it will panic
    pub fn write_to_current_ent(&mut self, ent: &FATDirEnt) {
        if self.inode.write_at_block_cache_lock(
            &mut self.inode_lock,
            self.offset.unwrap() as usize,
            ent.as_bytes(),
        ) != ent.as_bytes().len()
        {
            panic!("failed!");
        }
    }
    /// Internal implementation of iterator
    /// Depending on the direction, the offset tries to move a `FATDirEnt` distance
    /// See `DirIter` for move detail
    /// # Return Value
    /// If successful, it will return a `FATDirEnt`
    /// Otherwise, it will return None
    fn step(&mut self) -> Option<FATDirEnt> {
        let mut dir_ent: FATDirEnt = FATDirEnt::empty();
        if self.direction {
            // offset = None    => 0
            // otherwise        => offset + STEP_SIZE
            let offset = self.offset.map(|offset| offset + STEP_SIZE).unwrap_or(0);
            if offset >= self.inode.get_file_size_wlock(self.inode_lock) {
                return None;
            }
            self.inode.read_at_block_cache_wlock(
                &self.inode_lock,
                offset as usize,
                dir_ent.as_bytes_mut(),
            );
            match self.mode {
                DirIterMode::Enum => (),
                _ => {
                    // if directory entry is "last and unused", next is unavailable
                    if dir_ent.last_and_unused() {
                        return None;
                    }
                }
            }
            self.offset = Some(offset);
        } else {
            if self.offset.is_none() {
                return None;
            }
            if self.offset.unwrap() == 0 {
                self.offset = None;
                return None;
            }
            self.offset = self.offset.map(|offset| offset - STEP_SIZE);
            self.inode.read_at_block_cache_wlock(
                &self.inode_lock,
                self.offset.unwrap() as usize,
                dir_ent.as_bytes_mut(),
            );
        }
        Some(dir_ent)
    }
    /// Constructor for `DirWalker`
    /// # Return Value
    /// An `DirWalker`
    pub fn walk(self) -> DirWalker<'a, 'b> {
        matches!(self.mode, DirIterMode::Used);
        DirWalker { iter: self }
    }
}

/// Iterator for DirIter
/// `next()` will return the next valid entry and change to the offset corresponding to the target entry
/// `next()` will return None and will not change if the iterator is out of bounds with the next iterator
/// For modes other than `Enum`, their bounds are the `last_and_unused` entry or the start/end of the file
/// For `Enum` mode, its bounds are the start/end of the file(it will not be bound by the `last_and_unused` entry)
impl Iterator for DirIter<'_, '_> {
    type Item = FATDirEnt;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(dir_ent) = self.step() {
            fn check_dir_ent_legality(mode: &DirIterMode, dir_ent: &FATDirEnt) -> bool {
                match mode {
                    DirIterMode::Unused => dir_ent.unused_not_last(),
                    DirIterMode::Used => !dir_ent.unused(),
                    DirIterMode::Long => !dir_ent.unused() && dir_ent.is_long(),
                    DirIterMode::Short => !dir_ent.unused() && dir_ent.is_short(),
                    DirIterMode::Enum => true,
                }
            }
            if check_dir_ent_legality(&self.mode, &dir_ent) {
                return Some(dir_ent);
            }
        }
        None
    }
}

/// `DirWalker`: an iterator for directory file
/// It is based on `DirIter` and used to iterate over directory entries (combination of long and short entries)
pub struct DirWalker<'a, 'b> {
    pub iter: DirIter<'a, 'b>,
}

/// Iterator for DirWalker
impl Iterator for DirWalker<'_, '_> {
    type Item = (String, FATShortDirEnt);
    fn next(&mut self) -> Option<Self::Item> {
        let mut name = String::new();
        let mut should_be_ord = usize::MAX;
        while let Some(dir_ent) = self.iter.next() {
            if dir_ent.is_long() {
                if dir_ent.is_last_long_dir_ent() {
                    name = dir_ent.get_name() + &name;
                    should_be_ord = dir_ent.ord() - 1;
                } else if dir_ent.ord() == should_be_ord {
                    name = dir_ent.get_name() + &name;
                    should_be_ord -= 1;
                } else {
                    unreachable!()
                }
            } else if dir_ent.is_short() {
                if name.is_empty() {
                    name = dir_ent.get_name();
                }
                return Some((name, dir_ent.get_short_ent().unwrap().clone()));
            }
        }
        None
    }
}
