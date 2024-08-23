use crate::block::CName;
use lwext4_sys::ext4::*;
use alloc::string::{String, ToString};
use crate::error::{errno_to_result, Error, Result};
use crate::types::{FileAttr, FileTimes, Metadata, OpenFlags, Permissions, Time};
pub struct DirectoryNode {
    pub dir: ext4_dir,
    pub file: ext4_file,
    pub path: CName,
    pub is_file: bool
}

impl DirectoryNode {
    pub fn new(dir: ext4_dir, file: ext4_file, path: CName, is_file: bool) -> Self{
        Self { dir, file, path, is_file }
    }

    pub fn is_dir(&self) -> bool{
        !self.is_file
    }

    /// Set the file size
    pub fn set_len(&mut self, size: u64) -> Result<()> {
        unsafe {
            errno_to_result(ext4_ftruncate(&mut self.file as _, size))?;
        }
        Ok(())
    }

    /// Get the path of the directory entry
    pub fn get_path(&self) -> String {
        self.path.as_str().to_string()
    }

    /// Modify the times of a file
    pub fn set_times(&mut self, times: FileTimes) -> Result<()> {
        if let Some(a) = times.accessed {
            unsafe {
                errno_to_result(ext4_atime_set(self.path.as_ptr(), a.into()))?;
            }
        }
        if let Some(m) = times.modified {
            unsafe {
                errno_to_result(ext4_mtime_set(self.path.as_ptr(), m.into()))?;
            }
        }
        if let Some(c) = times.created {
            unsafe {
                errno_to_result(ext4_ctime_set(self.path.as_ptr(), c.into()))?;
            }
        }
        Ok(())
    }
}