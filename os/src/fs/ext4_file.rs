use super::{file_trait::File, DiskInodeType};

impl File for lwext4_rs::File {
    fn deep_clone(&self) -> alloc::sync::Arc<dyn File> {
        todo!()
    }

    fn readable(&self) -> bool {
        todo!()
    }

    fn writable(&self) -> bool {
        todo!()
    }

    fn read(&self, offset: Option<&mut usize>, buf: &mut [u8]) -> usize {
        todo!()
    }

    fn write(&self, offset: Option<&mut usize>, buf: &[u8]) -> usize {
        todo!()
    }

    fn r_ready(&self) -> bool {
        todo!()
    }

    fn w_ready(&self) -> bool {
        todo!()
    }

    fn read_user(&self, offset: Option<usize>, buf: crate::mm::UserBuffer) -> usize {
        todo!()
    }

    fn write_user(&self, offset: Option<usize>, buf: crate::mm::UserBuffer) -> usize {
        todo!()
    }

    fn get_size(&self) -> usize {
        todo!()
    }

    fn get_stat(&self) -> super::Stat {
        todo!()
    }

    fn get_statx(&self) -> super::Statx {
        todo!()
    }

    fn get_file_type(&self) -> DiskInodeType {
        todo!()
    }

    fn info_dirtree_node(
        &self,
        dirnode_ptr: alloc::sync::Weak<super::directory_tree::DirectoryTreeNode>,
    ) {
        todo!()
    }

    fn get_dirtree_node(
        &self,
    ) -> Option<alloc::sync::Arc<super::directory_tree::DirectoryTreeNode>> {
        todo!()
    }

    fn open(&self, flags: super::OpenFlags, special_use: bool) -> alloc::sync::Arc<dyn File> {
        todo!()
    }

    fn open_subfile(
        &self,
    ) -> Result<alloc::vec::Vec<(alloc::string::String, alloc::sync::Arc<dyn File>)>, isize> {
        todo!()
    }

    fn create(
        &self,
        name: &str,
        file_type: DiskInodeType,
    ) -> Result<alloc::sync::Arc<dyn File>, isize> {
        todo!()
    }

    fn link_child(&self, name: &str, child: &Self) -> Result<(), isize>
    where
        Self: Sized,
    {
        todo!()
    }

    fn unlink(&self, delete: bool) -> Result<(), isize> {
        todo!()
    }

    fn get_dirent(&self, count: usize) -> alloc::vec::Vec<super::Dirent> {
        todo!()
    }

    fn lseek(&self, offset: isize, whence: super::SeekWhence) -> Result<usize, isize> {
        todo!()
    }

    fn modify_size(&self, diff: isize) -> Result<(), isize> {
        todo!()
    }

    fn truncate_size(&self, new_size: usize) -> Result<(), isize> {
        todo!()
    }

    fn set_timestamp(&self, ctime: Option<usize>, atime: Option<usize>, mtime: Option<usize>) {
        todo!()
    }

    fn get_single_cache(
        &self,
        offset: usize,
    ) -> Result<alloc::sync::Arc<spin::Mutex<super::cache::PageCache>>, ()> {
        todo!()
    }

    fn get_all_caches(
        &self,
    ) -> Result<alloc::vec::Vec<alloc::sync::Arc<spin::Mutex<super::cache::PageCache>>>, ()> {
        todo!()
    }

    fn oom(&self) -> usize {
        todo!()
    }

    fn hang_up(&self) -> bool {
        todo!()
    }

    fn fcntl(&self, cmd: u32, arg: u32) -> isize {
        todo!()
    }
}
