use super::fat32::DiskInodeType;
use crate::{mm::UserBuffer, syscall::errno::ENOTTY};
use __alloc::string::String;
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use downcast_rs::*;
use spin::Mutex;

use super::{cache::PageCache, directory_tree::DirectoryTreeNode, layout::*};

pub trait File: DowncastSync {
    fn deep_clone(&self) -> Arc<dyn File>;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, offset: Option<&mut usize>, buf: &mut [u8]) -> usize;
    fn write(&self, offset: Option<&mut usize>, buf: &[u8]) -> usize;
    fn r_ready(&self) -> bool;
    fn w_ready(&self) -> bool;
    fn read_user(&self, offset: Option<usize>, buf: UserBuffer) -> usize;
    fn write_user(&self, offset: Option<usize>, buf: UserBuffer) -> usize;
    fn get_size(&self) -> usize;
    fn get_stat(&self) -> Stat;
    fn get_file_type(&self) -> DiskInodeType;
    fn is_dir(&self) -> bool {
        self.get_file_type() == DiskInodeType::Directory
    }
    fn is_file(&self) -> bool {
        self.get_file_type() == DiskInodeType::File
    }
    fn info_dirtree_node(&self, dirnode_ptr: Weak<DirectoryTreeNode>);
    fn get_dirtree_node(&self) -> Option<Arc<DirectoryTreeNode>>;
    /// open
    fn open(&self, flags: OpenFlags, special_use: bool) -> Arc<dyn File>;
    fn open_subfile(&self) -> Result<Vec<(String, Arc<dyn File>)>, isize>;
    /// create
    fn create(&self, name: &str, file_type: DiskInodeType) -> Result<Arc<dyn File>, isize>;
    fn link_child(&self, name: &str, child: &Self) -> Result<(), isize>
    where
        Self: Sized;
    /// delete(unlink)
    fn unlink(&self, delete: bool) -> Result<(), isize>;
    /// dirent
    fn get_dirent(&self, count: usize) -> Vec<Dirent>;
    /// offset
    fn get_offset(&self) -> usize {
        self.lseek(0, SeekWhence::SEEK_CUR).unwrap()
    }
    fn lseek(&self, offset: isize, whence: SeekWhence) -> Result<usize, isize>;
    /// size
    fn modify_size(&self, diff: isize) -> Result<(), isize>;
    fn truncate_size(&self, new_size: usize) -> Result<(), isize>;
    // time
    fn set_timestamp(&self, ctime: Option<usize>, atime: Option<usize>, mtime: Option<usize>);
    /// cache
    fn get_single_cache(&self, offset: usize) -> Result<Arc<Mutex<PageCache>>, ()>;
    fn get_all_caches(&self) -> Result<Vec<Arc<Mutex<PageCache>>>, ()>;
    /// memory related
    fn oom(&self) -> usize;
    /// poll, select related
    fn hang_up(&self) -> bool;
    /// iotcl
    fn ioctl(&self, _cmd: u32, _argp: usize) -> isize {
        ENOTTY
    }
    /// fcntl
    fn fcntl(&self, cmd: u32, arg: u32) -> isize;
}
impl_downcast!(sync File);
