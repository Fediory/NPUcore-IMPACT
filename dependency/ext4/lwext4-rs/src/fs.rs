use crate::block::CName;
use crate::dir::ReadDir;
use crate::error::{errno_to_result, Error, Result};
use crate::file::{raw_metadata, OpenOptions};
use crate::types::{FileType, Metadata, Permissions};
use crate::{BlockDeviceInterface, FileTimes, MetaDataExt, MountHandle, Time};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ptr::null_mut;
use log::info;
use lwext4_sys::ext4::*;
pub struct FileSystem<T: BlockDeviceInterface> {
    mp: MountHandle<T>,
}

impl<T: BlockDeviceInterface> Drop for FileSystem<T> {
    fn drop(&mut self) {
        info!("disable cache and stop journal");
        unsafe {
            errno_to_result(ext4_cache_write_back(self.mp.mount_point.as_ptr(), false)).unwrap();
            errno_to_result(ext4_journal_stop(self.mp.mount_point.as_ptr())).unwrap();
        }
    }
}

impl<T: BlockDeviceInterface> FileSystem<T> {
    pub fn new(mp: MountHandle<T>) -> Result<Self> {
        unsafe {
            errno_to_result(ext4_journal_start(mp.mount_point.as_ptr()))?;
            errno_to_result(ext4_cache_write_back(mp.mount_point.as_ptr(), true))?;
        }
        Ok(FileSystem { mp })
    }

    /// Get the mount point of the file system
    pub fn mount_handle(&self) -> &MountHandle<T> {
        &self.mp
    }

    /// Create a OpenOptions builder for opening a file
    pub fn file_builder(&self) -> OpenOptions {
        OpenOptions::new()
    }

    /// Get the metadata of a file
    pub fn metadata<P: AsRef<str>>(&self, path: P) -> Result<Metadata> {
        let path = CName::new(path.as_ref().to_string())?;
        raw_metadata(&path)
    }

    /// Open a directory at the provided path
    pub fn readdir<P: AsRef<str>>(&self, path: P) -> Result<ReadDir> {
        let mut raw_dir = ext4_dir {
            f: ext4_file {
                mp: null_mut(),
                inode: 0,
                flags: 0,
                fsize: 0,
                fpos: 0,
            },
            de: ext4_direntry {
                inode: 0,
                entry_length: 0,
                name_length: 0,
                inode_type: 0,
                name: [0u8; 255],
            },
            next_off: 0,
        };
        let path = CName::new(path.as_ref().to_string())?;
        unsafe {
            errno_to_result(ext4_dir_open(&mut raw_dir as _, path.as_ptr()))?;
        }
        Ok(ReadDir { raw: raw_dir, path })
    }

    /// Remove the file at the provided path
    pub fn remove_file<P: AsRef<str>>(&self, path: P) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_fremove(path.as_ptr())) }
    }

    /// Create a hard link at the provided path which points to the original file.
    pub fn hard_link<P: AsRef<str>, Q: AsRef<str>>(&self, original: P, link: Q) -> Result<()> {
        let original = CName::new(original.as_ref().to_string())?;
        let link = CName::new(link.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_flink(original.as_ptr(), link.as_ptr())) }
    }

    /// Create a new, empty directory at the provided path
    ///
    /// It will create all intermediate-level directories if they do not exist.
    pub fn create_dir<P: AsRef<str>>(&self, path: P) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_dir_mk(path.as_ptr())) }
    }

    /// It is the same as [create_dir](#method.create_dir)
    pub fn create_dir_all<P: AsRef<str>>(&self, path: P) -> Result<()> {
        self.create_dir(path)
    }

    /// Remove a directory at this path, after removing all its contents.
    pub fn remove_dir<P: AsRef<str>>(&self, path: P) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_dir_rm(path.as_ptr())) }
    }

    /// It is the same as [remove_dir](#method.remove_dir)
    pub fn remove_dir_all<P: AsRef<str>>(&self, path: P) -> Result<()> {
        self.remove_dir(path)
    }

    /// Rename a file or directory to a new name, replacing the original file if to already exists.
    pub fn rename<P: AsRef<str>, Q: AsRef<str>>(&self, from: P, to: Q) -> Result<()> {
        let from_cs = CName::new(from.as_ref().to_string())?;
        let to_cs = CName::new(to.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_frename(from_cs.as_ptr(), to_cs.as_ptr())) }
    }

    /// Get the target of a symbolic link
    pub fn read_link<P: AsRef<str>>(&self, path: P) -> Result<String> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut buf = [0u8; 255];
        let mut read = 0usize;
        unsafe {
            errno_to_result(ext4_readlink(
                path.as_ptr(),
                buf.as_mut_ptr() as _,
                buf.len(),
                &mut read as _,
            ))?;
        }
        Ok(String::from_utf8_lossy(&buf[..read]).to_string())
    }

    /// Set the permissions of a file or directory
    ///
    /// # Example
    /// ```no_run
    /// use lwext4_rs::Permissions;
    /// let p = Permissions::from_mode(0o777);
    /// // fs.set_permissions("/mp/hello.txt", p);
    /// ```
    pub fn set_permissions<P: AsRef<str>>(&self, path: P, perm: Permissions) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_mode_set(path.as_ptr(), perm.0)) }
    }

    /// Check if a file or directory exists at this path.
    pub fn exists<P: AsRef<str>>(&self, path: P) -> Result<bool> {
        let path = CName::new(path.as_ref().to_string())?;
        let res = unsafe { errno_to_result(ext4_inode_exist(path.as_ptr(), 0)) };
        match res {
            Ok(_) => Ok(true),
            Err(e) => match e {
                Error::NoEntry => Ok(false),
                _ => Err(e),
            },
        }
    }

    /// Create a symbolic link which points to the original file.
    pub fn soft_link<P: AsRef<str>, Q: AsRef<str>>(&self, original: P, link: Q) -> Result<()> {
        let original = CName::new(original.as_ref().to_string())?;
        let link = CName::new(link.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_fsymlink(original.as_ptr(), link.as_ptr())) }
    }

    /// Get the extended attribute of a file
    pub fn get_xattr<P: AsRef<str>, Q: AsRef<str>>(&self, path: P, name: Q) -> Result<Vec<u8>> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut buf = [0u8; 255];
        let mut read = 0usize;
        unsafe {
            errno_to_result(ext4_getxattr(
                path.as_ptr(),
                name.as_ref().as_ptr() as _,
                name.as_ref().len(),
                buf.as_mut_ptr() as _,
                buf.len(),
                &mut read as _,
            ))?;
        }
        Ok(buf[..read].to_vec())
    }

    /// Set an extended attribute of a file
    pub fn set_xattr<P: AsRef<str>, Q: AsRef<str>>(
        &self,
        path: P,
        name: Q,
        value: &[u8],
    ) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe {
            errno_to_result(ext4_setxattr(
                path.as_ptr(),
                name.as_ref().as_ptr() as _,
                name.as_ref().len(),
                value.as_ptr() as _,
                value.len(),
            ))?;
        }
        Ok(())
    }

    /// List all extended attributes of a file
    pub fn list_xattr<P: AsRef<str>>(&self, path: P) -> Result<Vec<Vec<u8>>> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut buf = Vec::<u8>::with_capacity(255);
        let mut read = 0usize;
        loop {
            unsafe {
                errno_to_result(ext4_listxattr(
                    path.as_ptr(),
                    buf.as_mut_ptr() as _,
                    buf.len(),
                    &mut read as _,
                ))?;
            }
            if read <= buf.len() {
                break;
            }
            buf.resize(buf.len() + 255, 0);
        }
        let mut res = Vec::new();

        buf.split(|&x| x == 0).for_each(|x| {
            if x.len() > 0 {
                res.push(x.to_vec());
            }
        });
        Ok(res)
    }

    /// Remove an extended attribute.
    ///
    /// Warning: This function is not available
    #[allow(unused)]
    fn remove_xattr<P: AsRef<str>, Q: AsRef<str>>(&self, path: P, name: Q) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe {
            errno_to_result(ext4_removexattr(
                path.as_ptr(),
                name.as_ref().as_ptr() as _,
                name.as_ref().len(),
            ))?;
        }
        Ok(())
    }

    /// Create a device node at the provided path
    ///
    /// The ty must not be regular file, directory, or unknown type.If filetype is char device or block device,
    /// the device number will become the payload in the inode.
    pub fn mknod<P: AsRef<str>>(&self, path: P, ty: FileType, dev: u32) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_mknod(path.as_ptr(), ty.to_ext4() as _, dev)) }
    }

    /// Change the owner and group of the specified path.
    ///
    /// Specifying either the uid or gid as None will leave it unchanged.
    pub fn chown<P: AsRef<str>>(&self, path: P, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        let (uid, gid) = if uid.is_none() && gid.is_none() {
            let meta = raw_metadata(&path)?;
            let uid = if uid.is_none() { Some(meta.uid()) } else { uid };
            let gid = if gid.is_none() { Some(meta.gid()) } else { gid };
            (uid, gid)
        } else {
            (uid, gid)
        };
        assert!(uid.is_some() && gid.is_some());
        unsafe { errno_to_result(ext4_owner_set(path.as_ptr(), uid.unwrap(), gid.unwrap())) }
    }
    /// Modify the times of a file
    pub fn set_times<P: AsRef<str>>(&self, path: P, times: FileTimes) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        if let Some(a) = times.accessed {
            unsafe {
                errno_to_result(ext4_atime_set(path.as_ptr(), a.into()))?;
            }
        }
        if let Some(m) = times.modified {
            unsafe {
                errno_to_result(ext4_mtime_set(path.as_ptr(), m.into()))?;
            }
        }
        if let Some(c) = times.created {
            unsafe {
                errno_to_result(ext4_ctime_set(path.as_ptr(), c.into()))?;
            }
        }
        Ok(())
    }

    /// Set the modified time of a file
    pub fn set_modified<P: AsRef<str>>(&mut self, path: P, time: Time) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe {
            errno_to_result(ext4_mtime_set(path.as_ptr(), time.into()))?;
        }
        Ok(())
    }
}
