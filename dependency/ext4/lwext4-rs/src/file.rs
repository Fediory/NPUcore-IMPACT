use crate::block::CName;
use crate::error::{errno_to_result, Error, Result};
use crate::types::{FileAttr, FileTimes, Metadata, OpenFlags, Permissions, Time};
use alloc::string::String;
use alloc::string::ToString;
use core::ptr::null_mut;
use embedded_io::{ErrorType, Read, Seek, SeekFrom, Write};
use lwext4_sys::ext4::*;

#[derive(Clone, Debug)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    mode: u32,
}

impl OpenOptions {
    pub(super) fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            mode: 0o666,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }

    pub(super) fn get_access_mode(&self) -> Result<OpenFlags> {
        match (self.read, self.write, self.append) {
            (true, false, false) => Ok(OpenFlags::RDONLY),
            (false, true, false) => Ok(OpenFlags::WRONLY),
            (true, true, false) => Ok(OpenFlags::RDWR),
            (false, _, true) => Ok(OpenFlags::WRONLY | OpenFlags::APPEND),
            (true, _, true) => Ok(OpenFlags::RDWR | OpenFlags::APPEND),
            (false, false, false) => Err(Error::InvalidArgument),
        }
    }

    pub(super) fn get_creation_mode(&self) -> Result<OpenFlags> {
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err(Error::InvalidArgument);
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err(Error::InvalidArgument);
                }
            }
        }

        Ok(match (self.create, self.truncate, self.create_new) {
            (false, false, false) => OpenFlags::empty(),
            (true, false, false) => OpenFlags::CREAT,
            (false, true, false) => OpenFlags::TRUNC,
            (true, true, false) => OpenFlags::CREAT | OpenFlags::TRUNC,
            (_, _, true) => OpenFlags::CREAT | OpenFlags::EXCL,
        })
    }
}

pub struct File {
    raw: ext4_file,
    path: CName,
}

impl File {
    pub(super) fn new(raw: ext4_file, path: CName) -> Self {
        Self { raw, path }
    }
    /// Get the metadata of a file
    pub fn metadata(&self) -> Result<Metadata> {
        raw_metadata(&self.path)
    }

    /// Set the file size
    pub fn set_len(&mut self, size: u64) -> Result<()> {
        unsafe {
            errno_to_result(ext4_ftruncate(&mut self.raw as _, size))?;
        }
        Ok(())
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

    /// Set the modified time of a file
    pub fn set_modified(&mut self, time: Time) -> Result<()> {
        unsafe {
            errno_to_result(ext4_mtime_set(self.path.as_ptr(), time.into()))?;
        }
        Ok(())
    }

    /// Set the permissions of a file
    pub fn set_permissions(&mut self, perm: Permissions) -> Result<()> {
        unsafe {
            errno_to_result(ext4_mode_set(self.path.as_ptr(), perm.0))?;
        }
        Ok(())
    }

    /// Reset the file pointer to the beginning
    pub fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Get the file pointer position
    pub fn stream_position(&mut self) -> Result<u64> {
        let pos = unsafe { ext4_ftell(&mut self.raw as _) };
        assert_eq!(pos, self.raw.fpos);
        Ok(pos)
    }

    /// Get the file path
    pub fn path(&self) -> String {
        self.path.as_str().to_string()
    }
}

pub fn raw_metadata(path: &CName) -> Result<Metadata> {
    let mut meta = Metadata(FileAttr::empty());
    let mut inode_nm = 0u32;
    unsafe {
        ext4_raw_inode_fill(path.as_ptr(), &mut inode_nm as _, &mut meta.0.raw as _);
    }
    meta.set_ino(inode_nm as _);
    Ok(meta)
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            errno_to_result(ext4_fclose(&mut self.raw)).unwrap();
        }
    }
}

impl ErrorType for File {
    type Error = Error;
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (origin, offset) = match pos {
            SeekFrom::Start(offset) => (SEEK_SET, offset as i64),
            SeekFrom::End(offset) => (SEEK_END, offset),
            SeekFrom::Current(offset) => (SEEK_CUR, offset),
        };
        unsafe {
            errno_to_result(ext4_fseek(&mut self.raw as _, offset, origin))?;
        }
        Ok(self.raw.fpos)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let mut read = 0usize;
            let buf_size = buf.len();
            errno_to_result(ext4_fread(
                &mut self.raw as _,
                buf.as_mut_ptr() as _,
                buf_size,
                &mut read as _,
            ))?;
            Ok(read)
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            let mut wrote = 0usize;
            let buf_size = buf.len();
            errno_to_result(ext4_fwrite(
                &mut self.raw as _,
                buf.as_ptr() as _,
                buf_size,
                &mut wrote as _,
            ))?;
            Ok(wrote)
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe { errno_to_result(ext4_cache_flush(self.path.as_ptr()))? }
        Ok(())
    }
}

impl OpenOptions {
    pub fn open<P: AsRef<str>>(&self, path: P) -> Result<File> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut raw_file = ext4_file {
            mp: null_mut(),
            inode: 0,
            flags: 0,
            fsize: 0,
            fpos: 0,
        };
        let flags = self.get_access_mode()? | self.get_creation_mode()?;
        unsafe {
            errno_to_result(ext4_fopen2(&mut raw_file, path.as_ptr(), flags.bits() as _))?;
        }
        // set mode
        if self.mode != 0o666 {
            unsafe {
                errno_to_result(ext4_mode_set(path.as_ptr(), self.mode))?;
            }
        }
        Ok(File {
            raw: raw_file,
            path,
        })
    }
}
