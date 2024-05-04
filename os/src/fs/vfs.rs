// #![allow(unused)]
// use super::dir_iter::*;
// use super::layout::{FATDirEnt, FATDiskInodeType, FATLongDirEnt, FATShortDirEnt};
use super::cache::{Cache, PageCache, PageCacheManager};
// use super::{DiskInodeType, EasyFileSystem};
type DiskInodeType = lwext4_rs::FileType;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::ops::Mul;
use core::panic;
use spin::*;

pub struct FileContent {
    /// For FAT32, size is a value computed from FAT.
    /// You should iterate around the FAT32 to get the size.
    size: u32,
    /// The cluster list.
    clus_list: Vec<u32>,
    /// If this file is a directory, hint will record the position of last directory entry(the first byte is 0x00).
    hint: u32,
}

impl FileContent {
    /// Get file size
    /// # Return Value
    /// The file size
    #[inline(always)]
    pub fn get_file_size(&self) -> u32 {
        self.size
    }
}
macro_rules! div_ceil {
    ($mult:expr,$deno:expr) => {
        ($mult - 1 + $deno) / $deno
    };
}
pub struct InodeTime {
    create_time: u64,
    access_time: u64,
    modify_time: u64,
}
#[allow(unused)]
impl InodeTime {
    /// Set the inode time's create time.
    pub fn set_create_time(&mut self, create_time: u64) {
        self.create_time = create_time;
    }

    /// Get a reference to the inode time's create time.
    pub fn create_time(&self) -> &u64 {
        &self.create_time
    }

    /// Set the inode time's access time.
    pub fn set_access_time(&mut self, access_time: u64) {
        self.access_time = access_time;
    }

    /// Get a reference to the inode time's access time.
    pub fn access_time(&self) -> &u64 {
        &self.access_time
    }

    /// Set the inode time's modify time.
    pub fn set_modify_time(&mut self, modify_time: u64) {
        self.modify_time = modify_time;
    }

    /// Get a reference to the inode time's modify time.
    pub fn modify_time(&self) -> &u64 {
        &self.modify_time
    }
}

pub struct InodeLock;
/* *ClusLi was DiskInode*
 * Even old New York, was New Amsterdam...
 * Why they changed it I can't say.
 * People just like it better that way.*/
/// The functionality of ClusLi & Inode can be merged.
/// The struct for file information
pub struct Inode {
    /// Inode lock: for normal operation
    inode_lock: RwLock<InodeLock>,
    /// File Content
    file_content: RwLock<FileContent>,
    /// File cache manager corresponding to this inode.
    file_cache_mgr: PageCacheManager,
    /// File type
    file_type: Mutex<DiskInodeType>,
    /// The parent directory of this inode
    parent_dir: Mutex<Option<(Arc<Self>, u32)>>,
    /// file system
    fs: Arc<EasyFileSystem>,
    /// Struct to hold time related information
    time: Mutex<InodeTime>,
    /// Info Inode to delete file content
    deleted: Mutex<bool>,
}

impl Drop for Inode {
    /// Before deleting the inode, the file information should be written back to the parent directory
    fn drop(&mut self) {
        if *self.deleted.lock() {
            // Clear size
            let mut lock = self.file_content.write();
            let length = lock.clus_list.len();
            self.dealloc_clus(&mut lock, length);
        } else {
            if self.parent_dir.lock().is_none() {
                return;
            }
            let par_dir_lock = self.parent_dir.lock();
            let (parent_dir, offset) = par_dir_lock.as_ref().unwrap();

            let par_inode_lock = parent_dir.write();
            let dir_ent = parent_dir.get_dir_ent(&par_inode_lock, *offset).unwrap();
            let mut short_dir_ent = *dir_ent.get_short_ent().unwrap();
            // Modify size
            short_dir_ent.file_size = self.get_file_size();
            // Modify fst cluster
            short_dir_ent.set_fst_clus(
                self.get_first_clus_lock(&self.file_content.read())
                    .unwrap_or(0),
            );
            // Modify time
            // todo!
            log::debug!("[Inode drop]: new_ent: {:?}", short_dir_ent);
            // Write back
            parent_dir
                .set_dir_ent(&par_inode_lock, *offset, dir_ent)
                .unwrap();
        }
    }
}

/// Constructor
impl Inode {
    /// Constructor for Inodes
    /// # Arguments
    /// + `fst_clus`: The first cluster of the file
    /// + `file_type`: The type of the inode determined by the file
    /// + `size`: NOTE: the `size` field should be set to `None` for a directory
    /// + `parent_dir`: parent directory
    /// + `fs`: The pointer to the file system
    /// # Return Value
    /// Pointer to Inode
    pub fn new(
        fst_clus: u32,
        file_type: DiskInodeType,
        size: Option<u32>,
        parent_dir: Option<(Arc<Self>, u32)>,
        fs: Arc<EasyFileSystem>,
    ) -> Arc<Self> {
        let file_cache_mgr = PageCacheManager::new();
        let clus_list = match fst_clus {
            0 => Vec::new(),
            _ => fs.fat.get_all_clus_num(fst_clus, &fs.block_device),
        };

        let size = size.unwrap_or_else(|| clus_list.len() as u32 * fs.clus_size());
        let hint = 0;

        let file_content = RwLock::new(FileContent {
            size,
            clus_list,
            hint,
        });
        let parent_dir = Mutex::new(parent_dir);
        let time = InodeTime {
            create_time: 0,
            access_time: 0,
            modify_time: 0,
        };
        let inode = Arc::new(Inode {
            inode_lock: RwLock::new(InodeLock {}),
            file_content,
            file_cache_mgr,
            file_type: Mutex::new(file_type),
            parent_dir,
            fs,
            time: Mutex::new(time),
            deleted: Mutex::new(false),
        });

        // Init hint
        if file_type.is_dir() {
            inode.set_hint();
        }
        inode
    }
}

/// Basic Funtions
impl Inode {
    /// Get self's file content lock
    /// # Return Value
    /// a lock of file content
    #[inline(always)]
    pub fn read(&self) -> RwLockReadGuard<InodeLock> {
        self.inode_lock.read()
    }
    #[inline(always)]
    pub fn write(&self) -> RwLockWriteGuard<InodeLock> {
        self.inode_lock.write()
    }
    pub fn get_file_type_lock(&self) -> MutexGuard<DiskInodeType> {
        self.file_type.lock()
    }
    /// Get file type
    pub fn get_file_type(&self) -> DiskInodeType {
        *self.file_type.lock()
    }
    #[inline(always)]
    pub fn get_file_size_rlock(&self, _inode_lock: &RwLockReadGuard<InodeLock>) -> u32 {
        self.get_file_size()
    }
    pub fn get_file_size_wlock(&self, _inode_lock: &RwLockWriteGuard<InodeLock>) -> u32 {
        self.get_file_size()
    }
    #[inline(always)]
    pub fn get_file_size(&self) -> u32 {
        self.file_content.read().get_file_size()
    }
    /// Check if file type is directory
    /// # Return Value
    /// Bool result
    #[inline(always)]
    pub fn is_dir(&self) -> bool {
        self.get_file_type().is_dir()
        // self.get_file_type() == DiskInodeType::Directory
    }
    /// Check if file type is file
    /// # Return Value
    /// Bool result
    #[inline(always)]
    pub fn is_file(&self) -> bool {
        self.get_file_type().is_file()
        // self.get_file_type() == DiskInodeType::File
    }
    /// Get first cluster of inode.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// # Return Value
    /// If cluster list isn't empty, it will return the first cluster list number.
    /// Otherwise it will return None.
    fn get_first_clus_lock(&self, lock: &RwLockReadGuard<FileContent>) -> Option<u32> {
        let clus_list = &lock.clus_list;
        if !clus_list.is_empty() {
            Some(clus_list[0])
        } else {
            None
        }
    }
    /// Get inode number of inode.
    /// For convenience, treat the first sector number as the inode number.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// # Return Value
    /// If cluster list isn't empty, it will return the first sector number.
    /// Otherwise it will return None.
    #[inline(always)]
    fn get_inode_num_lock(&self, lock: &RwLockReadGuard<FileContent>) -> Option<u32> {
        self.get_first_clus_lock(lock)
            .map(|clus| self.fs.first_sector_of_cluster(clus))
    }
    /// Get the number of clusters needed after rounding up according to size.
    /// # Return Value
    /// The number representing the number of clusters
    fn total_clus(&self, size: u32) -> u32 {
        //size.div_ceil(self.fs.clus_size())
        let clus_sz = self.fs.clus_size();
        div_ceil!(size, clus_sz)
        //(size - 1 + clus_sz) / clus_sz
    }
    /// Get first block id corresponding to the inner cache index
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// + `inner_cache_id`: The index of inner cache
    /// # Return Value
    /// If `inner_cache_id` is valid, it will return the first block id
    /// Otherwise it will return None
    #[inline(always)]
    fn get_block_id(
        &self,
        lock: &RwLockReadGuard<FileContent>,
        inner_cache_id: u32,
    ) -> Option<u32> {
        let idx = inner_cache_id as usize / self.fs.sec_per_clus as usize;
        let clus_list = &lock.clus_list;
        if idx >= clus_list.len() {
            return None;
        }
        let base = self.fs.first_sector_of_cluster(clus_list[idx]);
        let offset = inner_cache_id % self.fs.sec_per_clus as u32;
        Some(base + offset)
    }
    /// Get a list of `block_id` represented by the given cache index.
    /// # Arguments
    /// + `clus_list`: The cluster list
    /// + `inner_cache_id`: Index of T's file caches (usually 4096 size per cache)
    /// # Return Value
    /// List of `block_id`
    fn get_neighboring_sec(&self, clus_list: &Vec<u32>, inner_cache_id: usize) -> Vec<usize> {
        let sec_per_clus = self.fs.sec_per_clus as usize;
        let byts_per_sec = self.fs.byts_per_sec as usize;
        let sec_per_cache = PageCacheManager::CACHE_SZ / byts_per_sec;
        let mut sec_id = inner_cache_id * sec_per_cache;
        let mut block_ids = Vec::with_capacity(sec_per_cache);
        for _ in 0..sec_per_cache {
            let cluster_id = sec_id / sec_per_clus;
            if cluster_id >= clus_list.len() {
                break;
            }
            let offset = sec_id % sec_per_clus;
            let start_block_id = self.fs.first_sector_of_cluster(clus_list[cluster_id]) as usize;
            block_ids.push(start_block_id + offset);
            sec_id += 1;
        }
        block_ids
    }
    /// Open the root directory
    /// # Arguments
    /// + `efs`: The pointer to inner file system
    /// # Return Value
    /// A pointer to Inode
    pub fn root_inode(efs: &Arc<EasyFileSystem>) -> Arc<Self> {
        let rt_clus = efs.root_clus;
        Self::new(
            rt_clus,
            DiskInodeType::Directory,
            None,
            None,
            Arc::clone(efs),
        )
    }
}

/// File Content Operation
impl Inode {
    /// Allocate the required cluster.
    /// It will allocate as much as possible and then append to `clus_list` in `lock`.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// + `alloc_num`: Required number of clusters
    fn alloc_clus(&self, lock: &mut RwLockWriteGuard<FileContent>, alloc_num: usize) {
        let clus_list = &mut lock.clus_list;
        let mut new_clus_list = self.fs.fat.alloc(
            &self.fs.block_device,
            alloc_num,
            clus_list.last().map(|clus| *clus),
        );
        clus_list.append(&mut new_clus_list);
    }
    /// Release a certain number of clusters from `clus_list` in `lock`.
    /// `clus_list` will be emptied when the quantity to be freed exceeds the available quantity.
    /// # Arguments
    /// + `lock`: The lock of target file content
    /// + `dealloc_num`: The number of clusters that need to be released
    fn dealloc_clus(&self, lock: &mut RwLockWriteGuard<FileContent>, dealloc_num: usize) {
        let clus_list = &mut lock.clus_list;
        let dealloc_num = dealloc_num.min(clus_list.len());
        let mut dealloc_list = Vec::<u32>::with_capacity(dealloc_num);
        for _ in 0..dealloc_num {
            dealloc_list.push(clus_list.pop().unwrap());
        }
        self.fs.fat.free(
            &self.fs.block_device,
            dealloc_list,
            clus_list.last().map(|x| *x),
        );
    }
    /// Change the size of current file.
    /// This operation is ignored if the result size is negative
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `diff`: The change in file size
    /// # Warning
    /// This function will not modify its parent directory (since we changed the size of the current file),
    /// we will modify it when it is deleted.
    pub fn modify_size_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        diff: isize,
        clear: bool,
    ) {
        let mut lock = self.file_content.write();

        debug_assert!(diff.saturating_add(lock.size as isize) >= 0);

        let old_size = lock.size;
        let new_size = (lock.size as isize + diff) as u32;

        let old_clus_num = self.total_clus(old_size) as usize;
        let new_clus_num = self.total_clus(new_size) as usize;

        if diff > 0 {
            self.alloc_clus(&mut lock, new_clus_num - old_clus_num);
        } else {
            self.dealloc_clus(&mut lock, old_clus_num - new_clus_num);
        }

        lock.size = new_size;
        drop(lock);

        if diff > 0 {
            if clear {
                self.clear_at_block_cache_lock(
                    inode_lock,
                    old_size as usize,
                    (new_size - old_size) as usize,
                );
            }
        } else {
            self.file_cache_mgr.notify_new_size(new_size as usize)
        }
    }

    fn clear_at_block_cache_lock(
        &self,
        _inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: usize,
        length: usize,
    ) -> usize {
        let mut start = offset;
        let end = offset + length;

        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut write_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let lock = self.file_content.read();
            let block_write_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .modify(0, |data_block: &mut [u8; 4096]| {
                    let dst = &mut data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_write_size];
                    dst.fill(0);
                });
            drop(lock);
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        write_size
    }

    /// When memory is low, it is called to free its cache
    /// it just tries to lock it's file contents to free memory
    /// # Return Value
    /// The number of freed pages
    pub fn oom(&self) -> usize {
        let neighbor = |inner_cache_id| {
            self.get_neighboring_sec(&self.file_content.read().clus_list, inner_cache_id)
        };
        self.file_cache_mgr.oom(neighbor, &self.fs.block_device)
    }
}

/// IO
impl Inode {
    /// Read file content into buffer.
    /// It will read from `offset` until the end of the file or buffer can't read more
    /// This operation is ignored if start is greater than or equal to end.
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to receive data
    /// # Return Value
    /// The number of number of bytes read.
    pub fn read_at_block_cache_rlock(
        &self,
        _inode_lock: &RwLockReadGuard<InodeLock>,
        offset: usize,
        buf: &mut [u8],
    ) -> usize {
        let mut start = offset;
        let size = self.file_content.read().size as usize;
        let end = (offset + buf.len()).min(size);
        if start >= end {
            return 0;
        }
        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut read_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let lock = self.file_content.read();
            let block_read_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .read(0, |data_block: &[u8; 4096]| {
                    let dst = &mut buf[read_size..read_size + block_read_size];
                    let src = &data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            drop(lock);
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        read_size
    }
    /// do same thing but params different
    pub fn read_at_block_cache_wlock(
        &self,
        _inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: usize,
        buf: &mut [u8],
    ) -> usize {
        let mut start = offset;
        let size = self.file_content.read().size as usize;
        let end = (offset + buf.len()).min(size);
        if start >= end {
            return 0;
        }
        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut read_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let lock = self.file_content.read();
            let block_read_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .read(0, |data_block: &[u8; 4096]| {
                    let dst = &mut buf[read_size..read_size + block_read_size];
                    let src = &data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            drop(lock);
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        read_size
    }
    /// Read file content into buffer.
    /// It will read from `offset` until the end of the file or buffer can't read more
    /// This operation is ignored if start is greater than or equal to end.
    /// # Arguments    
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to receive data
    /// # Return Value
    /// The number of number of bytes read.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    #[inline(always)]
    pub fn read_at_block_cache(&self, offset: usize, buf: &mut [u8]) -> usize {
        self.read_at_block_cache_rlock(&self.read(), offset, buf)
    }

    /// Write buffer into file content.
    /// It will start to write from `offset` until the buffer is written,
    /// and when the write exceeds the end of file, it will modify file's size.
    /// If hard disk space id low, it will try to write as much data as possible.
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to write data
    /// # Return Value
    /// The number of number of bytes write.
    pub fn write_at_block_cache_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: usize,
        buf: &[u8],
    ) -> usize {
        let mut start = offset;
        let old_size = self.get_file_size() as usize;
        let diff_len = buf.len() as isize + offset as isize - old_size as isize;
        if diff_len > 0 as isize {
            // allocate as many clusters as possible.
            self.modify_size_lock(inode_lock, diff_len, false);
        }
        let end = (offset + buf.len()).min(self.get_file_size() as usize);

        debug_assert!(start <= end);

        let mut start_cache = start / PageCacheManager::CACHE_SZ;
        let mut write_size = 0;
        loop {
            // calculate end of current block
            let mut end_current_block =
                (start / PageCacheManager::CACHE_SZ + 1) * PageCacheManager::CACHE_SZ;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let lock = self.file_content.read();
            let block_write_size = end_current_block - start;
            self.file_cache_mgr
                .get_cache(
                    start_cache,
                    || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, start_cache) },
                    &self.fs.block_device,
                )
                .lock()
                // I know hardcoding 4096 in is bad, but I can't get around Rust's syntax checking...
                .modify(0, |data_block: &mut [u8; 4096]| {
                    let src = &buf[write_size..write_size + block_write_size];
                    let dst = &mut data_block[start % PageCacheManager::CACHE_SZ
                        ..start % PageCacheManager::CACHE_SZ + block_write_size];
                    dst.copy_from_slice(src);
                });
            drop(lock);
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_cache += 1;
            start = end_current_block;
        }
        write_size
    }

    /// Write buffer into file content.
    /// It will start to write from `offset` until the buffer is written,
    /// and when the write exceeds the end of file, it will modify file's size.
    /// If hard disk space id low, it will try to write as much data as possible.
    /// # Arguments    
    /// + `offset`: The start offset in file
    /// + `buf`: The buffer to write data
    /// # Return Value
    /// The number of number of bytes write.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    #[inline(always)]
    pub fn write_at_block_cache(&self, offset: usize, buf: &[u8]) -> usize {
        self.write_at_block_cache_lock(&self.write(), offset, buf)
    }

    /// Get a page cache corresponding to `inner_cache_id`.
    /// # Arguments    
    /// + `inner_cache_id`: The index of inner cache
    /// # Return Value
    /// Pointer to page cache
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    pub fn get_single_cache(&self, inner_cache_id: usize) -> Arc<Mutex<PageCache>> {
        self.get_single_cache_lock(&self.read(), inner_cache_id)
    }

    /// Get a page cache corresponding to `inner_cache_id`.
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `inner_cache_id`: The index of inner cache
    /// # Return Value
    /// Pointer to page cache
    pub fn get_single_cache_lock(
        &self,
        _inode_lock: &RwLockReadGuard<InodeLock>,
        inner_cache_id: usize,
    ) -> Arc<Mutex<PageCache>> {
        let lock = self.file_content.read();
        self.file_cache_mgr.get_cache(
            inner_cache_id,
            || -> Vec<usize> { self.get_neighboring_sec(&lock.clus_list, inner_cache_id) },
            &self.fs.block_device,
        )
    }

    /// Get all page caches corresponding to file
    /// # Return Value
    /// List of pointers to the page cache
    pub fn get_all_cache(&self) -> Vec<Arc<Mutex<PageCache>>> {
        let inode_lock = self.read();
        let lock = self.file_content.read();
        let cache_num =
            (lock.size as usize + PageCacheManager::CACHE_SZ - 1) / PageCacheManager::CACHE_SZ;
        let mut cache_list = Vec::<Arc<Mutex<PageCache>>>::with_capacity(cache_num);
        for inner_cache_id in 0..cache_num {
            cache_list.push(self.get_single_cache_lock(&inode_lock, inner_cache_id));
        }
        cache_list
    }
}

/// Directory Operation
impl Inode {
    /// A Constructor for `DirIter`(See `dir_iter.rs/DirIter` for details).
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The start offset of iterator
    /// + `mode`: The mode of iterator
    /// + `forward`: The direction of the iterator iteration
    /// # Return Value
    /// Pointer to iterator
    fn dir_iter<'a, 'b>(
        &'a self,
        inode_lock: &'a RwLockWriteGuard<'b, InodeLock>,
        offset: Option<u32>,
        mode: DirIterMode,
        forward: bool,
    ) -> DirIter<'a, 'b> {
        debug_assert!(self.is_dir(), "this isn't a directory");
        DirIter::new(inode_lock, offset, mode, forward, self)
    }
    /// Set the offset of the last entry in the directory file(first byte is 0x00) to hint
    fn set_hint(&self) {
        let inode_lock = self.write();
        let mut iter = self.dir_iter(&inode_lock, None, DirIterMode::Enum, FORWARD);
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                // Means iter reachs the end of file
                let mut lock = self.file_content.write();
                lock.hint = lock.size;
                return;
            }
            let dir_ent = dir_ent.unwrap();
            if dir_ent.last_and_unused() {
                let mut lock = self.file_content.write();
                lock.hint = iter.get_offset().unwrap();
                return;
            }
        }
    }
    /// Check if current file is an empty directory
    /// If a file contains only "." and "..", we consider it to be an empty directory
    /// # Arguments    
    /// + `inode_lock`: The lock of inode
    /// # Return Value
    /// Bool result
    pub fn is_empty_dir_lock(&self, inode_lock: &RwLockWriteGuard<InodeLock>) -> bool {
        if !self.is_dir() {
            return false;
        }
        let iter = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        for (name, _) in iter {
            if [".", ".."].contains(&name.as_str()) == false {
                return false;
            }
        }
        true
    }
    /// Expand directory file's size(a cluster)
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # Return Value
    /// Default is Ok
    fn expand_dir_size(&self, inode_lock: &RwLockWriteGuard<InodeLock>) -> Result<(), ()> {
        let diff_size = self.fs.clus_size();
        self.modify_size_lock(inode_lock, diff_size as isize, false);
        Ok(())
    }
    /// Shrink directory file's size to fit `hint`.
    /// For directory files, it has at least one cluster, which should be noted.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # Return Value
    /// Default is Ok
    fn shrink_dir_size(&self, inode_lock: &RwLockWriteGuard<InodeLock>) -> Result<(), ()> {
        let lock = self.file_content.read();
        let new_size = div_ceil!(lock.hint, self.fs.clus_size())
            .mul(self.fs.clus_size())
            .max(self.fs.clus_size());
        /*lock
        .hint
        .div_ceil(self.fs.clus_size())
        .mul(self.fs.clus_size())
        // For directory file, it has at least one cluster
        .max(self.fs.clus_size());*/
        let diff_size = new_size as isize - lock.size as isize;
        drop(lock);
        self.modify_size_lock(inode_lock, diff_size as isize, false);
        Ok(())
    }
    /// Allocate directory entries required for new file.
    /// The allocated directory entries is a contiguous segment.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `alloc_num`: Required number of directory entries
    /// # Return Value
    /// It will return lock anyway.
    /// If successful, it will also return the offset of the last allocated entry.
    fn alloc_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        alloc_num: usize,
    ) -> Result<u32, ()> {
        let offset = self.file_content.read().hint;
        let mut iter = self.dir_iter(inode_lock, None, DirIterMode::Enum, FORWARD);
        iter.set_iter_offset(offset);
        let mut found_free_dir_ent = 0;
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                if self.expand_dir_size(&mut iter.inode_lock).is_err() {
                    log::error!("[alloc_dir_ent]expand directory size error");
                    return Err(());
                }
                continue;
            }
            // We assume that all entries after `hint` are valid
            // That's why we use `hint`. It can reduce the cost of iterating over used entries
            found_free_dir_ent += 1;
            if found_free_dir_ent >= alloc_num {
                let offset = iter.get_offset().unwrap();
                // Set hint
                // Set next entry to last_and_unused
                if iter.next().is_some() {
                    iter.write_to_current_ent(&FATDirEnt::unused_and_last_entry());
                    let mut lock = self.file_content.write();
                    lock.hint = iter.get_offset().unwrap();
                } else {
                    // Means iter reachs the end of file
                    let mut lock = self.file_content.write();
                    lock.hint = lock.size;
                }
                return Ok(offset);
            }
        }
    }
    /// Get a directory entries.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of entry
    /// # Return Value
    /// If successful, it will return a `FATDirEnt`(See `layout.rs/FATDirEnt` for details)
    /// Otherwise, it will return Error
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    fn get_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<FATDirEnt, ()> {
        let mut dir_ent = FATDirEnt::empty();
        if self.read_at_block_cache_wlock(inode_lock, offset as usize, dir_ent.as_bytes_mut())
            != dir_ent.as_bytes().len()
        {
            return Err(());
        }
        Ok(dir_ent)
    }
    /// Write the directory entry back to the file contents.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of file to write
    /// + `dir_ent`: The buffer needs to write back
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    fn set_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
        dir_ent: FATDirEnt,
    ) -> Result<(), ()> {
        if self.write_at_block_cache_lock(inode_lock, offset as usize, dir_ent.as_bytes())
            != dir_ent.as_bytes().len()
        {
            return Err(());
        }
        Ok(())
    }
    /// Get directory entries, including short and long entries
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of short entry
    /// # Return Value
    /// If successful, it returns a pair of a short directory entry and a long directory entry list.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    fn get_all_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<(FATShortDirEnt, Vec<FATLongDirEnt>), ()> {
        debug_assert!(self.is_dir());
        let short_ent: FATShortDirEnt;
        let mut long_ents = Vec::<FATLongDirEnt>::with_capacity(5);

        let mut iter = self.dir_iter(inode_lock, Some(offset), DirIterMode::Enum, BACKWARD);

        short_ent = *iter.current_clone().unwrap().get_short_ent().unwrap();

        // Check if this directory entry is only a short directory entry
        {
            let dir_ent = iter.next();
            // First directory entry
            if dir_ent.is_none() {
                return Ok((short_ent, long_ents));
            }
            let dir_ent = dir_ent.unwrap();
            // Short directory entry
            if !dir_ent.is_long() {
                return Ok((short_ent, long_ents));
            }
        }

        // Get long dir_ents
        loop {
            let dir_ent = iter.current_clone();
            if dir_ent.is_none() {
                return Err(());
            }
            let dir_ent = dir_ent.unwrap();
            if dir_ent.get_long_ent().is_none() {
                return Err(());
            }
            long_ents.push(*dir_ent.get_long_ent().unwrap());
            if dir_ent.is_last_long_dir_ent() {
                break;
            }
        }
        Ok((short_ent, long_ents))
    }
    /// Delete derectory entries, including short and long entries.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset`: The offset of short entry
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock.
    fn delete_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
    ) -> Result<(), ()> {
        debug_assert!(self.is_dir());
        let mut iter = self.dir_iter(inode_lock, Some(offset), DirIterMode::Used, BACKWARD);

        iter.write_to_current_ent(&FATDirEnt::unused_not_last_entry());
        // Check if this directory entry is only a short directory entry
        {
            let dir_ent = iter.next();
            // First directory entry
            if dir_ent.is_none() {
                return Ok(());
            }
            let dir_ent = dir_ent.unwrap();
            // Short directory entry
            if !dir_ent.is_long() {
                return Ok(());
            }
        }
        // Remove long dir_ents
        loop {
            let dir_ent = iter.current_clone();
            if dir_ent.is_none() {
                return Err(());
            }
            let dir_ent = dir_ent.unwrap();
            if !dir_ent.is_long() {
                return Err(());
            }
            iter.write_to_current_ent(&FATDirEnt::unused_not_last_entry());
            iter.next();
            if dir_ent.is_last_long_dir_ent() {
                break;
            }
        }
        // Modify hint
        // We use new iterate mode
        let mut iter = self.dir_iter(
            inode_lock,
            Some(self.file_content.read().hint),
            DirIterMode::Enum,
            BACKWARD,
        );
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                // Indicates that the file is empty
                self.file_content.write().hint = 0;
                break;
            }
            let dir_ent = dir_ent.unwrap();
            if dir_ent.unused() {
                self.file_content.write().hint = iter.get_offset().unwrap();
                iter.write_to_current_ent(&FATDirEnt::unused_and_last_entry());
            } else {
                // Represents `iter` pointer to a used entry
                break;
            }
        }
        // Modify file size
        self.shrink_dir_size(inode_lock)
    }
    /// Create new disk space for derectory entries, including short and long entries.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `short_ent`: short entry
    /// + `long_ents`: list of long entries
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock.
    fn create_dir_ent(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        short_ent: FATShortDirEnt,
        long_ents: Vec<FATLongDirEnt>,
    ) -> Result<u32, ()> {
        debug_assert!(self.is_dir());
        let short_ent_offset = match self.alloc_dir_ent(inode_lock, 1 + long_ents.len()) {
            Ok(offset) => offset,
            Err(_) => return Err(()),
        };
        // We have graranteed we have alloc enough entries
        // So we use Enum mode
        let mut iter = self.dir_iter(
            inode_lock,
            Some(short_ent_offset),
            DirIterMode::Enum,
            BACKWARD,
        );

        iter.write_to_current_ent(&FATDirEnt {
            short_entry: short_ent,
        });
        for long_ent in long_ents {
            iter.next();
            iter.write_to_current_ent(&FATDirEnt {
                long_entry: long_ent,
            });
        }
        Ok(short_ent_offset)
    }
    /// Modify current directory file's ".." directory entry
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `parent_dir_clus_num`: The first cluster number of the parent directory
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's `file_content`, may cause deadlock
    fn modify_parent_dir_entry(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        parent_dir_clus_num: u32,
    ) -> Result<(), ()> {
        debug_assert!(self.is_dir());
        let mut iter = self.dir_iter(inode_lock, None, DirIterMode::Used, FORWARD);
        loop {
            let dir_ent = iter.next();
            if dir_ent.is_none() {
                break;
            }
            let mut dir_ent = dir_ent.unwrap();
            if dir_ent.get_name() == ".." {
                dir_ent.set_fst_clus(parent_dir_clus_num);
                iter.write_to_current_ent(&dir_ent);
                return Ok(());
            }
        }
        Err(())
    }
}

/// Delete
impl Inode {
    /// Delete the short and the long entry of `self` from `parent_dir`
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock self's parent_dir, may cause deadlock
    fn delete_self_dir_ent(&self) -> Result<(), ()> {
        if let Some((par_inode, offset)) = &*self.parent_dir.lock() {
            return par_inode.delete_dir_ent(&par_inode.write(), *offset);
        }
        Err(())
    }
    /// Delete the file from the disk,
    /// This file doesn't be removed immediately(dropped)
    /// deallocating both the directory entries (whether long or short),
    /// and the occupied clusters.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `delete`: Signal of deleting the file content when inode is dropped
    /// # Return Value
    /// If successful, it will return Ok.
    /// Otherwise, it will return Error with error number.
    /// # Warning
    /// This function will lock trash's `file_content`, may cause deadlock
    /// Make sure Arc has a strong count of 1.
    /// Make sure all its caches are not held by anyone else.
    /// Make sure target directory file is empty.
    pub fn unlink_lock(
        &self,
        _inode_lock: &RwLockWriteGuard<InodeLock>,
        delete: bool,
    ) -> Result<(), isize> {
        log::debug!(
            "[delete_from_disk] inode: {:?}, type: {:?}",
            self.get_inode_num_lock(&self.file_content.read()),
            self.file_type
        );
        // Remove directory entries
        if self.parent_dir.lock().is_none() {
            return Ok(());
        }
        if self.delete_self_dir_ent().is_err() {
            panic!()
        }
        if delete {
            *self.deleted.lock() = true;
        }
        *self.parent_dir.lock() = None;
        Ok(())
    }
}

/// Link
impl Inode {
    pub fn link_par_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: String,
    ) -> Result<(), ()> {
        // Genrate directory entries
        let (short_ent, long_ents) = Self::gen_dir_ent(
            parent_dir,
            parent_inode_lock,
            &name,
            self.get_first_clus_lock(&self.file_content.read())
                .unwrap_or(0),
            *self.file_type.lock(),
        );
        // Allocate new directory entry
        let short_ent_offset =
            match parent_dir.create_dir_ent(parent_inode_lock, short_ent, long_ents) {
                Ok(offset) => offset,
                Err(_) => return Err(()),
            };
        // If this is a directory, modify ".."
        if self.is_dir()
            && self
                .modify_parent_dir_entry(
                    inode_lock,
                    parent_dir
                        .get_first_clus_lock(&parent_dir.file_content.read())
                        .unwrap(),
                )
                .is_err()
        {
            return Err(());
        }
        // Modify parent directory
        *self.parent_dir.lock() = Some((parent_dir.clone(), short_ent_offset));
        Ok(())
    }
}

/// Create
impl Inode {
    /// Create a file or a directory from the parent.
    /// The parent directory will write the new file directory entries.
    /// # Arguments
    /// + `parent_dir`: the pointer to parent directory inode
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: new file's name
    /// + `file_type`: new file's file type
    /// # Return Value
    /// If successful, it will return the new file inode
    /// Otherwise, it will return Error.
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    /// The length of name should be less than 256(for ascii), otherwise the file system can not store.
    /// Make sure there are no duplicate names in parent_dir.
    pub fn create_lock(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: String,
        file_type: DiskInodeType,
    ) -> Result<Arc<Self>, ()> {
        if parent_dir.is_file() || name.len() >= 256 {
            Err(())
        } else {
            log::debug!(
                "[create] par_inode: {:?}, name: {:?}, file_type: {:?}",
                parent_dir.get_inode_num_lock(&parent_dir.file_content.read()),
                &name,
                file_type
            );
            // If file_type is Directory, alloc first cluster
            let fst_clus = if file_type == DiskInodeType::Directory {
                let fst_clus = parent_dir
                    .fs
                    .fat
                    .alloc(&parent_dir.fs.block_device, 1, None);
                if fst_clus.is_empty() {
                    return Err(());
                }
                fst_clus[0]
            } else {
                0
            };
            // Genrate directory entries
            let (short_ent, long_ents) =
                Self::gen_dir_ent(parent_dir, parent_inode_lock, &name, fst_clus, file_type);
            // Create directory entry
            let short_ent_offset =
                match parent_dir.create_dir_ent(parent_inode_lock, short_ent, long_ents) {
                    Ok(offset) => offset,
                    Err(_) => return Err(()),
                };
            // Generate current file
            let current_file = Self::from_ent(&parent_dir, &short_ent, short_ent_offset);
            // If file_type is Directory, set first 3 directory entry
            if file_type == DiskInodeType::Directory {
                // Set hint
                current_file.file_content.write().hint =
                    2 * core::mem::size_of::<FATDirEnt>() as u32;
                // Fill content
                Self::fill_empty_dir(&parent_dir, &current_file, fst_clus);
            }
            Ok(current_file)
        }
    }

    /// Construct a \[u16,13\] corresponding to the `long_ent_num`'th 13-u16 or shorter name slice
    /// _NOTE_: the first entry is of number 0 for `long_ent_num`
    /// # Arguments
    /// + `name`: File name
    /// + `long_ent_index`: The index of long entry(start from 0)
    /// # Return Value
    /// A long name slice
    fn gen_long_name_slice(name: &String, long_ent_index: usize) -> [u16; 13] {
        let mut v: Vec<u16> = name.encode_utf16().collect();
        debug_assert!(long_ent_index * 13 < v.len());
        while v.len() < (long_ent_index + 1) * 13 {
            v.push(0);
        }
        let start = long_ent_index * 13;
        let end = (long_ent_index + 1) * 13;
        v[start..end].try_into().expect("should be able to cast")
    }

    /// Construct a \[u8,11\] corresponding to the short directory entry name
    /// # Arguments
    /// + `parent_dir`: The pointer to parent directory
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: File name
    /// # Return Value
    /// A short name slice
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    fn gen_short_name_slice(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: &String,
    ) -> [u8; 11] {
        let short_name = FATDirEnt::gen_short_name_prefix(name.clone());
        if short_name.len() == 0 || short_name.find(' ').unwrap_or(8) == 0 {
            panic!("illegal short name");
        }

        let mut short_name_slice = [0u8; 11];
        short_name_slice.copy_from_slice(&short_name.as_bytes()[0..11]);

        let iter = parent_dir.dir_iter(parent_inode_lock, None, DirIterMode::Short, FORWARD);
        FATDirEnt::gen_short_name_numtail(iter.collect(), &mut short_name_slice);
        short_name_slice
    }
    /// Construct short and long entries name slices
    /// # Arguments
    /// + `parent_dir`: The pointer to parent directory
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: File name
    /// # Return Value
    /// A pair of a short name slice and a list of long name slices
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    fn gen_name_slice(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: &String,
    ) -> ([u8; 11], Vec<[u16; 13]>) {
        let short_name_slice = Self::gen_short_name_slice(parent_dir, parent_inode_lock, name);

        let long_ent_num = div_ceil!(name.len(), 13);
        //name.len().div_ceil(13);
        let mut long_name_slices = Vec::<[u16; 13]>::with_capacity(long_ent_num);
        for i in 0..long_ent_num {
            long_name_slices.push(Self::gen_long_name_slice(name, i));
        }

        (short_name_slice, long_name_slices)
    }

    /// Construct short and long entries
    /// # Arguments
    /// + `parent_dir`: The pointer to parent directory
    /// + `parent_inode_lock`: the lock of parent's inode
    /// + `name`: File name
    /// + `fst_clus`: The first cluster of constructing file
    /// + `file_type`: The file type of constructing file
    /// # Return Value
    /// A pair of a short directory entry and a list of long name entries
    /// # Warning
    /// This function will lock the `file_content` of the parent directory, may cause deadlock
    fn gen_dir_ent(
        parent_dir: &Arc<Self>,
        parent_inode_lock: &RwLockWriteGuard<InodeLock>,
        name: &String,
        fst_clus: u32,
        file_type: DiskInodeType,
    ) -> (FATShortDirEnt, Vec<FATLongDirEnt>) {
        // Generate name slices
        let (short_name_slice, long_name_slices) =
            Self::gen_name_slice(parent_dir, parent_inode_lock, &name);
        // Generate short entry
        let short_ent = FATShortDirEnt::from_name(short_name_slice, fst_clus, file_type);
        // Generate long entries
        let long_ent_num = long_name_slices.len();
        let long_ents = long_name_slices
            .iter()
            .enumerate()
            .map(|(i, slice)| FATLongDirEnt::from_name_slice(i + 1 == long_ent_num, i + 1, *slice))
            .collect();
        (short_ent, long_ents)
    }

    /// Create a file from directory entry.
    /// # Arguments
    /// + `parent_dir`: the parent directory inode pointer
    /// + `ent`: the short entry as the source of information
    /// + `offset`: the offset of the short directory entry in the `parent_dir`
    /// # Return Value
    /// Pointer to Inode
    pub fn from_ent(parent_dir: &Arc<Self>, ent: &FATShortDirEnt, offset: u32) -> Arc<Self> {
        Self::new(
            ent.get_first_clus(),
            if ent.is_dir() {
                DiskInodeType::Directory
            } else {
                DiskInodeType::File
            },
            if ent.is_file() {
                Some(ent.file_size)
            } else {
                None
            },
            Some((parent_dir.clone(), offset)),
            parent_dir.fs.clone(),
        )
    }

    /// Fill out an empty directory with only the '.' & '..' entries.
    /// # Arguments
    /// + `parent_dir`: the pointer of parent directory inode
    /// + `current_dir`: the pointer of new directory inode
    /// + `fst_clus`: the first cluster number of current file
    fn fill_empty_dir(parent_dir: &Arc<Self>, current_dir: &Arc<Self>, fst_clus: u32) {
        let current_inode_lock = current_dir.write();
        let mut iter = current_dir.dir_iter(&current_inode_lock, None, DirIterMode::Enum, FORWARD);
        let mut short_name: [u8; 11] = [' ' as u8; 11];
        //.
        iter.next();
        short_name[0] = '.' as u8;
        iter.write_to_current_ent(&FATDirEnt {
            short_entry: FATShortDirEnt::from_name(
                short_name,
                fst_clus as u32,
                DiskInodeType::Directory,
            ),
        });
        //..
        iter.next();
        short_name[1] = '.' as u8;
        iter.write_to_current_ent(&FATDirEnt {
            short_entry: FATShortDirEnt::from_name(
                short_name,
                parent_dir
                    .get_first_clus_lock(&parent_dir.file_content.read())
                    .unwrap(),
                DiskInodeType::Directory,
            ),
        });
        //add "unused and last" sign
        iter.next();
        iter.write_to_current_ent(&FATDirEnt::unused_and_last_entry());
    }
}

// ls and find local
impl Inode {
    /// ls - General Purose file filterer
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # WARNING
    /// The definition of OFFSET is CHANGED for this item.
    /// It should point to the NEXT USED entry whether it as a long entry whenever possible or a short entry if no long ones exist.
    /// # Return value
    /// On success, the function returns `Ok(_)`. On failure, multiple chances exist: either the Vec is empty, or the Result is `Err(())`.
    /// # Implementation Information
    /// The iterator stops at the last available item when it reaches the end,
    /// returning `None` from then on,
    /// so relying on the offset of the last item to decide whether it has reached an end is not recommended.
    #[inline(always)]
    pub fn ls_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
    ) -> Result<Vec<(String, FATShortDirEnt)>, ()> {
        if !self.is_dir() {
            return Err(());
        }
        Ok(self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk()
            .collect())
    }
    /// find `req_name` in current directory file
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `req_name`: required file name
    /// # Return value
    /// On success, the function returns `Ok(_)`. On failure, multiple chances exist: either the Vec is empty, or the Result is `Err(())`.
    pub fn find_local_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        req_name: String,
    ) -> Result<Option<(String, FATShortDirEnt, u32)>, ()> {
        if !self.is_dir() {
            return Err(());
        }
        log::debug!("[find_local] name: {:?}", req_name);
        let mut walker = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        match walker.find(|(name, _)| {
            name.len() == req_name.len() && name.as_str().eq_ignore_ascii_case(req_name.as_str())
        }) {
            Some((name, short_ent)) => {
                log::trace!("[find_local] Query name: {} found", req_name);
                Ok(Some((name, short_ent, walker.iter.get_offset().unwrap())))
            }
            None => {
                log::trace!("[find_local] Query name: {} not found", req_name);
                Ok(None)
            }
        }
    }
}

// metadata
impl Inode {
    /// Return the `time` field of `self`
    pub fn time(&self) -> MutexGuard<InodeTime> {
        self.time.lock()
    }
    /// Return the `stat` structure to `self` file.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// # Return value
    /// (file size, access time, modify time, create time, inode number)
    pub fn stat_lock(&self, _inode_lock: &RwLockReadGuard<InodeLock>) -> (i64, i64, i64, i64, u64) {
        let time = self.time.lock();
        (
            self.get_file_size() as i64,
            time.access_time as i64,
            time.modify_time as i64,
            time.create_time as i64,
            self.get_inode_num_lock(&self.file_content.read())
                .unwrap_or(0) as u64,
        )
    }

    pub fn get_all_files_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
    ) -> Vec<(String, FATShortDirEnt, u32)> {
        let mut vec = Vec::with_capacity(8);
        let mut walker = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        loop {
            let ele = walker.next();
            match ele {
                Some((name, short_ent)) => {
                    if name == "." || name == ".." {
                        continue;
                    }
                    vec.push((name, short_ent, walker.iter.get_offset().unwrap()))
                }
                None => break,
            }
        }
        vec
    }

    /// Get a dirent information from the `self` at `offset`
    /// Return `None` if `self` is not a directory.
    /// # Arguments
    /// + `inode_lock`: The lock of inode
    /// + `offset` The offset within the `self` directory.
    /// + `length` The length of required vector
    /// # Return value
    /// On success, the function returns `Ok(file name, file size, first cluster, file type)`.
    /// On failure, multiple chances exist: either the Vec is empty, or the Result is `Err(())`.
    pub fn dirent_info_lock(
        &self,
        inode_lock: &RwLockWriteGuard<InodeLock>,
        offset: u32,
        length: usize,
    ) -> Result<Vec<(String, usize, u64, FATDiskInodeType)>, ()> {
        if !self.is_dir() {
            return Err(());
        }
        let size = self.get_file_size();
        let mut walker = self
            .dir_iter(inode_lock, None, DirIterMode::Used, FORWARD)
            .walk();
        walker.iter.set_iter_offset(offset);
        let mut v = Vec::with_capacity(length);

        let (mut last_name, mut last_short_ent) = match walker.next() {
            Some(tuple) => tuple,
            None => return Ok(v),
        };
        for _ in 0..length {
            let next_dirent_offset =
                walker.iter.get_offset().unwrap() as usize + core::mem::size_of::<FATDirEnt>();
            let (name, short_ent) = match walker.next() {
                Some(tuple) => tuple,
                None => {
                    v.push((
                        last_name,
                        size as usize,
                        last_short_ent.get_first_clus() as u64,
                        last_short_ent.attr,
                    ));
                    return Ok(v);
                }
            };
            v.push((
                last_name,
                next_dirent_offset,
                last_short_ent.get_first_clus() as u64,
                last_short_ent.attr,
            ));
            last_name = name;
            last_short_ent = short_ent;
        }
        Ok(v)
    }
}
