use crate::block::CName;
use crate::error::{Error, Result};
use crate::types::*;
use crate::File;
use alloc::string::{String, ToString};
use core::ffi::CStr;
use core::fmt::Debug;
use core::mem::transmute;
use lwext4_sys::ext4::*;

pub struct ReadDir {
    pub(super) raw: ext4_dir,
    pub(super) path: CName,
}

impl Debug for ReadDir {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReadDir").field("path", &self.path).finish()
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        unsafe {
            ext4_dir_close(&mut self.raw as _);
        }
    }
}

impl ReadDir {
    /// Reset the directory stream to the beginning.
    pub fn rewind(&mut self) {
        unsafe { ext4_dir_entry_rewind(&mut self.raw as _) }
    }
    /// Use the directory as file
    pub fn as_file(&self) -> File {
        File::new(self.raw.f.clone(), self.path.clone())
    }
    /// Get the directory path
    pub fn path(&self) -> String {
        self.path.as_str().to_string()
    }
}

pub struct DirEntry {
    raw: ext4_direntry,
    root: CName,
}

impl Debug for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DirEntry")
            .field("name", &self.name())
            .field("path", &self.path())
            .field("inode", &self.inode())
            .field("file_type", &self.file_type())
            .finish()
    }
}

impl DirEntry {
    /// Get the name of the directory entry
    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(&self.raw.name)
                .to_str()
                .unwrap()
                .trim_matches('\0')
        }
    }

    /// Get the path of the directory entry
    pub fn path(&self) -> String {
        self.root.as_str().to_string() + self.name()
    }

    /// Get the inode of the directory entry
    pub fn inode(&self) -> u32 {
        // isn't 64 bit inode supported?
        self.raw.inode
    }

    /// Get the file type of the directory entry
    pub fn file_type(&self) -> Result<FileType> {
        let ty = self.raw.inode_type;
        match ty as u32 {
            EXT4_DE_CHRDEV => Ok(FileType { mode: S_IFCHR }),
            EXT4_DE_FIFO => Ok(FileType { mode: S_IFIFO }),
            EXT4_DE_SYMLINK => Ok(FileType { mode: S_IFLNK }),
            EXT4_DE_REG_FILE => Ok(FileType { mode: S_IFREG }),
            EXT4_DE_SOCK => Ok(FileType { mode: S_IFSOCK }),
            EXT4_DE_DIR => Ok(FileType { mode: S_IFDIR }),
            EXT4_DE_BLKDEV => Ok(FileType { mode: S_IFBLK }),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl Iterator for ReadDir {
    type Item = DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let result = ext4_dir_entry_next(&mut self.raw as _);
            if result.is_null() {
                None
            } else {
                let res = DirEntry {
                    raw: (*transmute::<_, &ext4_direntry>(result)).clone(),
                    root: self.path.clone(),
                };
                Some(res)
            }
        }
    }
}
