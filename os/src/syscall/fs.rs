use crate::arch::BLOCK_SZ;
use crate::fs::poll::{ppoll, pselect, FdSet, PollFd};
use crate::fs::*;
use crate::mm::{
    copy_from_user, copy_from_user_array, copy_to_user, copy_to_user_array, copy_to_user_string,
    translated_byte_buffer, translated_byte_buffer_append_to_existing_vec, translated_refmut,
    translated_str, try_get_from_user, MapPermission, UserBuffer, VirtAddr,
};
use crate::task::{current_task, current_user_token};
use crate::timer::TimeSpec;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use log::{debug, info, trace, warn};
use num_enum::FromPrimitive;

use super::errno::*;

pub const AT_FDCWD: usize = 100usize.wrapping_neg();

/// # Warning
/// `fs` & `files` is locked in this function
fn __openat(dirfd: usize, path: &str) -> Result<FileDescriptor, isize> {
    let task = current_task().unwrap();
    let file_descriptor = match dirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return Err(errno),
            }
        }
    };
    file_descriptor.open(path, OpenFlags::O_RDONLY, false)
}

pub fn sys_getcwd(buf: usize, size: usize) -> isize {
    let task = current_task().unwrap();
    if !task
        .vm
        .lock()
        .contains_valid_buffer(buf, size, MapPermission::W)
    {
        // buf points to a bad address.
        return EFAULT;
    }
    if size == 0 && buf != 0 {
        // The size argument is zero and buf is not a NULL pointer.
        return EINVAL;
    }
    let working_dir = task.fs.lock().working_inode.get_cwd().unwrap();
    if working_dir.len() >= size {
        // The size argument is less than the length of the absolute pathname of the working directory,
        // including the terminating null byte.
        return ERANGE;
    }
    let token = task.get_user_token();
    UserBuffer::new({
        match translated_byte_buffer(token, buf as *const u8, size) {
            Ok(buffer) => buffer,
            Err(errno) => return errno,
        }
    })
    .write(working_dir.as_bytes());
    buf as isize
}

pub fn sys_lseek(fd: usize, offset: isize, whence: u32) -> isize {
    // whence is not valid
    let whence = match SeekWhence::from_bits(whence) {
        Some(whence) => whence,
        None => {
            warn!("[sys_lseek] unknown flags");
            return EINVAL;
        }
    };
    info!(
        "[sys_lseek] fd: {}, offset: {}, whence: {:?}",
        fd, offset, whence,
    );
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    match file_descriptor.lseek(offset, whence) {
        Ok(pos) => pos as isize,
        Err(errno) => errno,
    }
}

pub fn sys_read(fd: usize, buf: usize, count: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    // fd is not open for reading
    if !file_descriptor.readable() {
        return EBADF;
    }
    let token = task.get_user_token();
    file_descriptor.read_user(
        None,
        UserBuffer::new({
            match translated_byte_buffer(token, buf as *const u8, count) {
                Ok(buffer) => buffer,
                Err(errno) => return errno,
            }
        }),
    ) as isize
}

pub fn sys_write(fd: usize, buf: usize, count: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    if !file_descriptor.writable() {
        return EBADF;
    }
    let token = task.get_user_token();
    file_descriptor.write_user(
        None,
        UserBuffer::new({
            match translated_byte_buffer(token, buf as *const u8, count) {
                Ok(buffer) => buffer,
                Err(errno) => return errno,
            }
        }),
    ) as isize
}

pub fn sys_pread(fd: usize, buf: usize, count: usize, offset: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    // fd is not open for reading
    if !file_descriptor.readable() {
        return EBADF;
    }
    let token = task.get_user_token();
    file_descriptor.read_user(
        Some(offset),
        UserBuffer::new({
            match translated_byte_buffer(token, buf as *const u8, count) {
                Ok(buffer) => buffer,
                Err(errno) => return errno,
            }
        }),
    ) as isize
}

pub fn sys_pwrite(fd: usize, buf: usize, count: usize, offset: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    // fd is not open for writing
    if !file_descriptor.writable() {
        return EBADF;
    }
    let token = task.get_user_token();
    file_descriptor.write_user(
        Some(offset),
        UserBuffer::new({
            match translated_byte_buffer(token, buf as *const u8, count) {
                Ok(buffer) => buffer,
                Err(errno) => return errno,
            }
        }),
    ) as isize
}

#[repr(C)]
#[derive(Clone, Copy)]
struct IOVec {
    iov_base: *const u8, /* Starting address */
    iov_len: usize,      /* Number of bytes to transfer */
}

pub fn sys_readv(fd: usize, iov: usize, iovcnt: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    // fd is not open for reading
    if !file_descriptor.readable() {
        return EBADF;
    }
    let token = task.get_user_token();
    let mut iovecs = Vec::<IOVec>::with_capacity(iovcnt);
    if copy_from_user_array(token, iov as *const IOVec, iovecs.as_mut_ptr(), iovcnt).is_err() {
        // See read(2), which the ERRORS section of readv is written in addition to.
        log::error!("[readv] Failed to copy from {:?}", iov);
        return EFAULT;
    };
    unsafe { iovecs.set_len(iovcnt) };
    file_descriptor.read_user(
        None,
        UserBuffer::new({
            let mut vec = Vec::with_capacity(32);
            for iovec in iovecs.iter() {
                match translated_byte_buffer_append_to_existing_vec(
                    &mut vec,
                    token,
                    iovec.iov_base,
                    iovec.iov_len,
                ) {
                    Ok(_) => continue,
                    Err(errno) => return errno,
                }
            }
            vec
        }),
    ) as isize
}

pub fn sys_writev(fd: usize, iov: usize, iovcnt: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    // fd is not open for writing
    if !file_descriptor.writable() {
        return EBADF;
    }
    let token = task.get_user_token();
    let mut iovecs = Vec::<IOVec>::with_capacity(iovcnt);
    if copy_from_user_array(token, iov as *const IOVec, iovecs.as_mut_ptr(), iovcnt).is_err() {
        log::error!("[writev] Failed to copy from {:?}", iov);
        return EFAULT;
    };
    unsafe { iovecs.set_len(iovcnt) };
    file_descriptor.write_user(
        None,
        UserBuffer::new({
            let mut vec = Vec::with_capacity(32);
            for iovec in iovecs.iter() {
                match translated_byte_buffer_append_to_existing_vec(
                    &mut vec,
                    token,
                    iovec.iov_base,
                    iovec.iov_len,
                ) {
                    Ok(_) => continue,
                    Err(errno) => return errno,
                }
            }
            vec
        }),
    ) as isize
}

/// If offset is not NULL, then it points to a variable holding the
/// file offset from which sendfile() will start reading data from
/// in_fd.
///
/// When sendfile() returns,
/// this variable will be set to the offset of the byte following
/// the last byte that was read.
///
/// If offset is not NULL, then sendfile() does not modify the file
/// offset of in_fd; otherwise the file offset is adjusted to reflect
/// the number of bytes read from in_fd.
///
/// If offset is NULL, then data will be read from in_fd starting at
/// the file offset, and the file offset will be updated by the call.
pub fn sys_sendfile(out_fd: usize, in_fd: usize, offset: *mut usize, count: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let in_file = match fd_table.get_ref(in_fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    let out_file = match fd_table.get_ref(out_fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    info!("[sys_sendfile] outfd: {}, in_fd: {}", out_fd, in_fd);
    if !in_file.readable() || !out_file.writable() {
        return EBADF;
    }

    let token = task.get_user_token();
    // turn a pointer in user space into a pointer in kernel space if it is not null
    let offset = if offset.is_null() {
        offset
    } else {
        match translated_refmut(token, offset) {
            Ok(offset) => offset as *mut usize,
            Err(errno) => return errno,
        }
    };

    // a buffer in kernel
    const BUFFER_SIZE: usize = 4096;
    let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
    let mut buffer_ptr: Option<&[u8]> = None;

    let mut left_bytes = count;
    loop {
        let write_buffer = match buffer_ptr {
            Some(buffer_ptr) => buffer_ptr,
            None => {
                unsafe {
                    buffer.set_len(left_bytes.min(BUFFER_SIZE));
                }
                let read_size = in_file.read(unsafe { offset.as_mut() }, buffer.as_mut_slice());
                if (read_size as isize) < 0 {
                    let errno = read_size as isize;
                    return errno;
                } else if read_size == 0 {
                    break;
                }
                unsafe {
                    buffer.set_len(read_size);
                }
                buffer.as_slice()
            }
        };

        let read_size = write_buffer.len();

        let fallback = |redundant_bytes: usize| unsafe {
            let offset = offset.as_mut();
            match offset {
                Some(offset) => {
                    *offset -= redundant_bytes;
                }
                None => match in_file.lseek(-(redundant_bytes as isize), SeekWhence::SEEK_CUR) {
                    Ok(_) => {}
                    Err(errno) => panic!("failed! errno {}", errno),
                },
            }
        };

        let write_size = out_file.write(None, write_buffer);
        if (write_size as isize) < 0 {
            fallback(read_size);
            let errno = read_size as isize;
            return errno;
        } else if write_size == 0 {
            fallback(read_size);
            break;
        }

        buffer_ptr = if write_size < read_size {
            Some(&write_buffer[write_size..])
        } else {
            None
        };
        left_bytes -= write_size;
    }
    let send_size = count - left_bytes;
    info!("[sys_sendfile] send bytes: {}", send_size);
    send_size as isize
}

pub fn sys_close(fd: usize) -> isize {
    info!("[sys_close] fd: {}", fd);
    let task = current_task().unwrap();
    let mut fd_table = task.files.lock();
    match fd_table.remove(fd) {
        Ok(_) => SUCCESS,
        Err(errno) => errno,
    }
}

/// # Warning
/// Only O_CLOEXEC is supported now
pub fn sys_pipe2(pipefd: usize, flags: u32) -> isize {
    const VALID_FLAGS: OpenFlags = OpenFlags::from_bits_truncate(
        0o2000000 /* O_CLOEXEC */ | 0o40000 /* O_DIRECT */ | 0o4000, /* O_NONBLOCK */
    );
    let flags = match OpenFlags::from_bits(flags) {
        Some(flags) => {
            // only O_CLOEXEC | O_DIRECT | O_NONBLOCK are valid in pipe2()
            if flags.difference(VALID_FLAGS).is_empty() {
                flags
            } else {
                // some flags are invalid in pipe2(), they are all valid OpenFlags though
                warn!(
                    "[sys_pipe2] invalid flags: {:?}",
                    flags.difference(VALID_FLAGS)
                );
                return EINVAL;
            }
        }
        None => {
            // contains invalid OpenFlags
            warn!("[sys_pipe2] unknown flags");
            return EINVAL;
        }
    };
    let task = current_task().unwrap();
    let mut fd_table = task.files.lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = match fd_table.insert(FileDescriptor::new(
        flags.contains(OpenFlags::O_CLOEXEC),
        false,
        pipe_read,
    )) {
        Ok(fd) => fd,
        Err(errno) => return errno,
    };
    let write_fd = match fd_table.insert(FileDescriptor::new(
        flags.contains(OpenFlags::O_CLOEXEC),
        false,
        pipe_write,
    )) {
        Ok(fd) => fd,
        Err(errno) => return errno,
    };

    let token = task.get_user_token();
    if copy_to_user_array(
        token,
        [read_fd as u32, write_fd as u32].as_ptr(),
        pipefd as *mut u32,
        2,
    )
    .is_err()
    {
        log::error!("[sys_pipe2] Failed to copy to {:?}", pipefd);
        return EFAULT;
    };
    info!(
        "[sys_pipe2] read_fd: {}, write_fd: {}, flags: {:?}",
        read_fd, write_fd, flags
    );
    SUCCESS
}

pub fn sys_getdents64(fd: usize, dirp: *mut u8, count: usize) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();

    let file_descriptor = match fd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };
    let dirent_vec = match file_descriptor.get_dirent(count) {
        Ok(vec) => vec,
        Err(errno) => return errno,
    };
    if copy_to_user_array(
        token,
        dirent_vec.as_ptr(),
        dirp as *mut Dirent,
        dirent_vec.len(),
    )
    .is_err()
    {
        log::error!("[sys_getdents64] Failed to copy to {:?}", dirp);
        return EFAULT;
    };
    info!("[sys_getdents64] fd: {}, count: {}", fd, count);
    (dirent_vec.len() * size_of::<Dirent>()) as isize
}

pub fn sys_dup(oldfd: usize) -> isize {
    let task = current_task().unwrap();
    let mut fd_table = task.files.lock();
    let old_file_descriptor = match fd_table.get_ref(oldfd) {
        Ok(file_descriptor) => file_descriptor.clone(),
        Err(errno) => return errno,
    };
    let newfd = match fd_table.insert(old_file_descriptor) {
        Ok(fd) => fd,
        Err(errno) => return errno,
    };
    info!("[sys_dup] oldfd: {}, newfd: {}", oldfd, newfd);
    newfd as isize
}

pub fn sys_dup3(oldfd: usize, newfd: usize, flags: u32) -> isize {
    info!(
        "[sys_dup3] oldfd: {}, newfd: {}, flags: {:?}",
        oldfd,
        newfd,
        OpenFlags::from_bits(flags)
    );
    if oldfd == newfd {
        return EINVAL;
    }
    let is_cloexec = match OpenFlags::from_bits(flags) {
        Some(OpenFlags::O_CLOEXEC) => true,
        // `O_RDONLY == 0`, so it means *NO* cloexec in fact
        Some(OpenFlags::O_RDONLY) => false,
        // flags contain an invalid value
        Some(flags) => {
            warn!("[sys_dup3] invalid flags: {:?}", flags);
            return EINVAL;
        }
        None => {
            warn!("[sys_dup3] unknown flags");
            return EINVAL;
        }
    };
    let task = current_task().unwrap();
    let mut fd_table = task.files.lock();

    let mut file_descriptor = match fd_table.get_ref(oldfd) {
        Ok(file_descriptor) => file_descriptor.clone(),
        Err(errno) => return errno,
    };
    file_descriptor.set_cloexec(is_cloexec);
    match fd_table.insert_at(file_descriptor, newfd) {
        Ok(fd) => fd as isize,
        Err(errno) => errno,
    }
}

// This syscall is not complete at all, only /read proc/self/exe
pub fn sys_readlinkat(dirfd: usize, pathname: *const u8, buf: *mut u8, bufsiz: usize) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = match translated_str(token, pathname) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    let real_path = if path.as_str() == "/proc/self/exe" {
        task.exe.lock().get_cwd().unwrap()
    } else {
        match __openat(dirfd, &path) {
            Ok(_) => {
                // we don't implement symbolic link, so if we found it...
                warn!(
                    "[sys_readlinkat] not a symbolic link! dirfd: {}, path: {}",
                    dirfd as isize, path
                );
                // The file of `pathname` is not a symbolic link
                return EINVAL;
            }
            Err(errno) => return errno,
        }
    };
    let len = real_path.len().min(bufsiz - 1);
    // `copy_to_user_string` will add '\0' in the end, so written length is `len + 1`
    if copy_to_user_string(token, &real_path[0..len], buf).is_err() {
        log::error!("[sys_readlinkat] Failed to copy to {:?}", buf);
        return EFAULT;
    };

    debug!(
        "[sys_readlinkat] dirfd: {}, pathname: {}, buf: {:?}, bufsiz: {}, written: {}",
        dirfd as isize, path, buf, bufsiz, real_path
    );

    (len + 1) as isize
}

bitflags! {
    pub struct FstatatFlags: u32 {
        const AT_EMPTY_PATH = 0x1000;
        const AT_NO_AUTOMOUNT = 0x800;
        const AT_SYMLINK_NOFOLLOW = 0x100;
    }
}

pub fn sys_fstatat(dirfd: usize, path: *const u8, buf: *mut u8, flags: u32) -> isize {
    let token = current_user_token();
    let path = match translated_str(token, path) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    let flags = match FstatatFlags::from_bits(flags) {
        Some(flags) => flags,
        None => {
            warn!("[sys_fstatat] unknown flags");
            return EINVAL;
        }
    };

    info!(
        "[sys_fstatat] dirfd: {}, path: {:?}, flags: {:?}",
        dirfd as isize, path, flags,
    );

    let task = current_task().unwrap();
    let file_descriptor = match dirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };

    match file_descriptor.open(&path, OpenFlags::O_RDONLY, false) {
        Ok(file_descriptor) => {
            if copy_to_user(token, &file_descriptor.get_stat(), buf as *mut Stat).is_err() {
                log::error!("[sys_fstatat] Failed to copy to {:?}", buf);
                return EFAULT;
            };
            SUCCESS
        }
        Err(errno) => errno,
    }
}

pub fn sys_fstat(fd: usize, statbuf: *mut u8) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();

    info!("[sys_fstat] fd: {}", fd);
    let file_descriptor = match fd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };
    if copy_to_user(token, &file_descriptor.get_stat(), statbuf as *mut Stat).is_err() {
        log::error!("[sys_fstat] Failed to copy to {:?}", statbuf);
        return EFAULT;
    };
    SUCCESS
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Statfs {
    /// Type of filesystem
    f_type: usize,
    /// Optimal transfer block size
    f_bsize: usize,
    /// Total data blocks in filesystem
    f_blocks: u64,
    /// Free blocks in filesystem
    f_bfree: u64,
    /// Free blocks available to
    /// unprivileged user
    f_bavail: u64,
    /// Total file nodes in filesystem
    f_files: u64,
    /// Free file nodes in filesystem
    f_ffree: u64,
    /// Filesystem ID
    f_fsid: [i32; 2],
    /// Maximum length of filenames
    f_namelen: usize,
    /// Fragment size (since Linux 2.6)
    f_frsize: usize,
    /// Mount flags of filesystem
    f_flag: usize,
    /// Padding bytes reserved for future use
    f_spare: [usize; 4],
}
/// Fake implement for statfs syscall
pub fn sys_statfs(_path: *const u8, buf: *mut Statfs) -> isize {
    let statfs = Box::new(Statfs {
        f_type: 0xf2f52010,
        f_bsize: BLOCK_SZ,
        f_blocks: 10000,
        f_bfree: 9000,
        f_bavail: 9000,
        f_files: 1000,
        f_ffree: 960,
        f_fsid: [114, 514],
        f_namelen: 256,
        f_frsize: 0,
        f_flag: 0,
        f_spare: [0; 4],
    });
    let token = current_task().unwrap().get_user_token();
    if copy_to_user(token, statfs.as_ref(), buf).is_err() {
        log::error!("[sys_statfs] Failed to copy to {:?}", buf);
        return EFAULT;
    };
    SUCCESS
}

pub fn sys_fsync(fd: usize) -> isize {
    let task = current_task().unwrap();

    info!("[sys_fsync] fd: {}", fd);
    let fd_table = task.files.lock();
    if let Err(errno) = fd_table.check(fd) {
        return errno;
    }
    SUCCESS
}

pub fn sys_chdir(path: *const u8) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = match translated_str(token, path) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    info!("[sys_chdir] path: {}", path);

    let mut lock = task.fs.lock();

    match lock.working_inode.cd(&path) {
        Ok(new_working_inode) => {
            lock.working_inode = new_working_inode;
            SUCCESS
        }
        Err(errno) => errno,
    }
}

pub fn sys_openat(dirfd: usize, path: *const u8, flags: u32, mode: u32) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = match translated_str(token, path) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    let flags = match OpenFlags::from_bits(flags) {
        Some(flags) => flags,
        None => {
            warn!("[sys_openat] unknown flags");
            return EINVAL;
        }
    };
    let mode = StatMode::from_bits(mode);
    info!(
        "[sys_openat] dirfd: {}, path: {}, flags: {:?}, mode: {:?}",
        dirfd as isize, path, flags, mode
    );
    let mut fd_table = task.files.lock();
    let file_descriptor = match dirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => match fd_table.get_ref(fd) {
            Ok(file_descriptor) => file_descriptor.clone(),
            Err(errno) => return errno,
        },
    };

    let new_file_descriptor = match file_descriptor.open(&path, flags, false) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };

    let new_fd = match fd_table.insert(new_file_descriptor) {
        Ok(fd) => fd,
        Err(errno) => return errno,
    };
    new_fd as isize
}

pub fn sys_renameat2(
    olddirfd: usize,
    oldpath: *const u8,
    newdirfd: usize,
    newpath: *const u8,
    flags: u32,
) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let oldpath = match translated_str(token, oldpath) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    let newpath = match translated_str(token, newpath) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    info!(
        "[sys_renameat2] olddirfd: {}, oldpath: {}, newdirfd: {}, newpath: {}, flags: {}",
        olddirfd as isize, oldpath, newdirfd as isize, newpath, flags
    );

    let old_file_descriptor = match olddirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };
    let new_file_descriptor = match newdirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };

    match FileDescriptor::rename(
        &old_file_descriptor,
        &oldpath,
        &new_file_descriptor,
        &newpath,
    ) {
        Ok(_) => SUCCESS,
        Err(errno) => errno,
    }
}

pub fn sys_ioctl(fd: usize, cmd: u32, arg: usize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    file_descriptor.ioctl(cmd, arg)
}

pub fn sys_ppoll(fds: usize, nfds: usize, tmo_p: usize, sigmask: usize) -> isize {
    ppoll(
        fds as *mut PollFd,
        nfds,
        tmo_p as *const TimeSpec,
        sigmask as *const crate::task::Signals,
    )
}

pub fn sys_mkdirat(dirfd: usize, path: *const u8, mode: u32) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = match translated_str(token, path) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    info!(
        "[sys_mkdirat] dirfd: {}, path: {}, mode: {:?}",
        dirfd as isize,
        path,
        StatMode::from_bits(mode)
    );
    let file_descriptor = match dirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };
    match file_descriptor.mkdir(&path) {
        Ok(_) => SUCCESS,
        Err(errno) => errno,
    }
}

bitflags! {
    pub struct UnlinkatFlags: u32 {
        const AT_REMOVEDIR = 0x200;
    }
}

/// # Warning
/// Currently we have no hard-link so this syscall will remove file directly.
pub fn sys_unlinkat(dirfd: usize, path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = match translated_str(token, path) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    let flags = match UnlinkatFlags::from_bits(flags) {
        Some(flags) => flags,
        None => {
            warn!("[sys_unlinkat] unknown flags");
            return EINVAL;
        }
    };
    info!(
        "[sys_unlinkat] dirfd: {}, path: {}, flags: {:?}",
        dirfd as isize, path, flags
    );

    let file_descriptor = match dirfd {
        AT_FDCWD => task.fs.lock().working_inode.as_ref().clone(),
        fd => {
            let fd_table = task.files.lock();
            match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            }
        }
    };
    match file_descriptor.delete(&path, flags.contains(UnlinkatFlags::AT_REMOVEDIR)) {
        Ok(_) => SUCCESS,
        Err(errno) => errno,
    }
}

bitflags! {
    pub struct UmountFlags: u32 {
        const MNT_FORCE           =   1;
        const MNT_DETACH          =   2;
        const MNT_EXPIRE          =   4;
        const UMOUNT_NOFOLLOW     =   8;
    }
}

pub fn sys_umount2(target: *const u8, flags: u32) -> isize {
    if target.is_null() {
        return EINVAL;
    }
    let token = current_user_token();
    let target = match translated_str(token, target) {
        Ok(target) => target,
        Err(errno) => return errno,
    };
    let flags = match UmountFlags::from_bits(flags) {
        Some(flags) => flags,
        None => return EINVAL,
    };
    info!("[sys_umount2] target: {}, flags: {:?}", target, flags);
    warn!("[sys_umount2] fake implementation!");
    SUCCESS
}

bitflags! {
    pub struct MountFlags: usize {
        const MS_RDONLY         =   1;
        const MS_NOSUID         =   2;
        const MS_NODEV          =   4;
        const MS_NOEXEC         =   8;
        const MS_SYNCHRONOUS    =   16;
        const MS_REMOUNT        =   32;
        const MS_MANDLOCK       =   64;
        const MS_DIRSYNC        =   128;
        const MS_NOATIME        =   1024;
        const MS_NODIRATIME     =   2048;
        const MS_BIND           =   4096;
        const MS_MOVE           =   8192;
        const MS_REC            =   16384;
        const MS_SILENT         =   32768;
        const MS_POSIXACL       =   (1<<16);
        const MS_UNBINDABLE     =   (1<<17);
        const MS_PRIVATE        =   (1<<18);
        const MS_SLAVE          =   (1<<19);
        const MS_SHARED         =   (1<<20);
        const MS_RELATIME       =   (1<<21);
        const MS_KERNMOUNT      =   (1<<22);
        const MS_I_VERSION      =   (1<<23);
        const MS_STRICTATIME    =   (1<<24);
        const MS_LAZYTIME       =   (1<<25);
        const MS_NOREMOTELOCK   =   (1<<27);
        const MS_NOSEC          =   (1<<28);
        const MS_BORN           =   (1<<29);
        const MS_ACTIVE         =   (1<<30);
        const MS_NOUSER         =   (1<<31);
    }
}

pub fn sys_mount(
    source: *const u8,
    target: *const u8,
    filesystemtype: *const u8,
    mountflags: usize,
    data: *const u8,
) -> isize {
    if source.is_null() || target.is_null() || filesystemtype.is_null() {
        return EINVAL;
    }
    let token = current_user_token();
    let source = match translated_str(token, source) {
        Ok(source) => source,
        Err(errno) => return errno,
    };
    let target = match translated_str(token, target) {
        Ok(target) => target,
        Err(errno) => return errno,
    };
    let filesystemtype = match translated_str(token, filesystemtype) {
        Ok(filesystemtype) => filesystemtype,
        Err(errno) => return errno,
    };
    // infallible
    let mountflags = MountFlags::from_bits(mountflags).unwrap();
    info!(
        "[sys_mount] source: {}, target: {}, filesystemtype: {}, mountflags: {:?}, data: {:?}",
        source, target, filesystemtype, mountflags, data
    );
    warn!("[sys_mount] fake implementation!");
    SUCCESS
}

bitflags! {
    pub struct UtimensatFlags: u32 {
        const AT_SYMLINK_NOFOLLOW = 0x100;
    }
}

pub fn sys_utimensat(
    dirfd: usize,
    pathname: *const u8,
    times: *const [TimeSpec; 2],
    flags: u32,
) -> isize {
    const UTIME_NOW: usize = 0x3fffffff;
    const UTIME_OMIT: usize = 0x3ffffffe;
    let token = current_user_token();
    let path = if !pathname.is_null() {
        match translated_str(token, pathname) {
            Ok(path) => path,
            Err(errno) => return errno,
        }
    } else {
        String::new()
    };
    let flags = match UtimensatFlags::from_bits(flags) {
        Some(flags) => flags,
        None => {
            warn!("[sys_utimensat] unknown flags");
            return EINVAL;
        }
    };

    info!(
        "[sys_utimensat] dirfd: {}, path: {}, times: {:?}, flags: {:?}",
        dirfd as isize, path, times, flags
    );

    let inode = match __openat(dirfd, &path) {
        Ok(inode) => inode,
        Err(errno) => return errno,
    };

    let now = TimeSpec::now();
    let timespec = &mut [now; 2];
    let mut atime = Some(now.tv_sec);
    let mut mtime = Some(now.tv_sec);
    if !times.is_null() {
        if copy_from_user(token, times, timespec).is_err() {
            log::error!("[sys_utimensat] Failed to copy from {:?}", times);
            return EFAULT;
        };
        match timespec[0].tv_nsec {
            UTIME_NOW => (),
            UTIME_OMIT => atime = None,
            _ => atime = Some(timespec[0].tv_sec),
        }
        match timespec[1].tv_nsec {
            UTIME_NOW => (),
            UTIME_OMIT => mtime = None,
            _ => mtime = Some(timespec[1].tv_sec),
        }
    }

    inode.set_timestamp(None, atime, mtime).unwrap();
    SUCCESS
}

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, FromPrimitive)]
#[repr(u32)]
pub enum Fcntl_Command {
    DUPFD = 0,
    GETFD = 1,
    SETFD = 2,
    GETFL = 3,
    SETFL = 4,
    GETLK = 5,
    SETLK = 6,
    SETLKW = 7,
    SETOWN = 8,
    GETOWN = 9,
    SETSIG = 10,
    GETSIG = 11,
    SETOWN_EX = 15,
    GETOWN_EX = 16,
    GETOWNER_UIDS = 17,
    OFD_GETLK = 36,
    OFD_SETLK = 37,
    OFD_SETLKW = 38,
    SETLEASE = 1024,
    GETLEASE = 1025,
    NOTIFY = 1026,
    CANCELLK = 1029,
    DUPFD_CLOEXEC = 1030,
    SETPIPE_SZ = 1031,
    GETPIPE_SZ = 1032,
    ADD_SEALS = 1033,
    GET_SEALS = 1034,
    GET_RW_HINT = 1035,
    SET_RW_HINT = 1036,
    GET_FILE_RW_HINT = 1037,
    SET_FILE_RW_HINT = 1038,
    #[num_enum(default)]
    ILLEAGAL,
}

pub fn sys_fcntl(fd: usize, cmd: u32, arg: usize) -> isize {
    const FD_CLOEXEC: usize = 1;

    let task = current_task().unwrap();
    let mut fd_table = task.files.lock();

    info!(
        "[sys_fcntl] fd: {}, cmd: {:?}, arg: {:X}",
        fd,
        Fcntl_Command::from_primitive(cmd),
        arg
    );

    match Fcntl_Command::from_primitive(cmd) {
        Fcntl_Command::DUPFD => {
            let new_file_descriptor = match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            };
            match fd_table.try_insert_at(new_file_descriptor, arg) {
                Ok(fd) => fd as isize,
                Err(errno) => errno,
            }
        }
        Fcntl_Command::DUPFD_CLOEXEC => {
            let mut new_file_descriptor = match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor.clone(),
                Err(errno) => return errno,
            };
            new_file_descriptor.set_cloexec(true);
            match fd_table.try_insert_at(new_file_descriptor, arg) {
                Ok(fd) => fd as isize,
                Err(errno) => errno,
            }
        }
        Fcntl_Command::GETFD => {
            let file_descriptor = match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor,
                Err(errno) => return errno,
            };
            file_descriptor.get_cloexec() as isize
        }
        Fcntl_Command::SETFD => {
            let file_descriptor = match fd_table.get_refmut(fd) {
                Ok(file_descriptor) => file_descriptor,
                Err(errno) => return errno,
            };
            file_descriptor.set_cloexec((arg & FD_CLOEXEC) != 0);
            if (arg & !FD_CLOEXEC) != 0 {
                warn!("[fcntl] Unsupported flag exists: {:X}", arg);
            }
            SUCCESS
        }
        Fcntl_Command::GETFL => {
            let file_descriptor = match fd_table.get_ref(fd) {
                Ok(file_descriptor) => file_descriptor,
                Err(errno) => return errno,
            };
            // Access control is not fully implemented
            let mut res = OpenFlags::O_RDWR.bits() as isize;
            if file_descriptor.get_nonblock() {
                res |= OpenFlags::O_NONBLOCK.bits() as isize;
            }
            res
        }
        command => {
            warn!("[fcntl] Unsupported command: {:?}", command);
            SUCCESS
        } // WARNING!!!
    }
}

pub fn sys_pselect(
    nfds: usize,
    read_fds: *mut FdSet,
    write_fds: *mut FdSet,
    exception_fds: *mut FdSet,
    timeout: *mut TimeSpec,
    sigmask: *const crate::task::signal::Signals,
) -> isize {
    if (nfds as isize) < 0 {
        return EINVAL;
    }
    let token = current_user_token();
    let mut kread_fds = match try_get_from_user(token, read_fds) {
        Ok(fds) => fds,
        Err(errno) => return errno,
    };
    let mut kwrite_fds = match try_get_from_user(token, write_fds) {
        Ok(fds) => fds,
        Err(errno) => return errno,
    };
    let mut kexception_fds = match try_get_from_user(token, exception_fds) {
        Ok(fds) => fds,
        Err(errno) => return errno,
    };
    let ktimeout = match try_get_from_user(token, timeout) {
        Ok(timeout) => timeout,
        Err(errno) => return errno,
    };
    let mut ret = pselect(
        nfds,
        &mut kread_fds,
        &mut kwrite_fds,
        &mut kexception_fds,
        &ktimeout,
        sigmask,
    );
    /*
    WARNING! The EFAULT errno is NOT mentioned in man for Linux.
    However, it is mentioned in BSD man, so we keep it anyway.
     */
    if let Some(kread_fds) = &kread_fds {
        trace!("[pselect] read_fds: {:?}", kread_fds);
        if copy_to_user(token, kread_fds, read_fds).is_err() {
            log::error!("[sys_pselect] Error copying to read_fds {:?}", read_fds);
            ret = EFAULT;
        };
    }
    if let Some(kwrite_fds) = &kwrite_fds {
        trace!("[pselect] write_fds: {:?}", kwrite_fds);
        if copy_to_user(token, kwrite_fds, write_fds).is_err() {
            log::error!("[sys_pselect] Error copying to write_fds {:?}", write_fds);
            ret = EFAULT;
        };
    }
    if let Some(kexception_fds) = &kexception_fds {
        trace!("[pselect] exception_fds: {:?}", kexception_fds);
        if copy_to_user(token, kexception_fds, exception_fds).is_err() {
            log::error!(
                "[sys_pselect] Error copying to exception_fds {:?}",
                exception_fds
            );
            ret = EFAULT;
        };
    }
    ret
}

/// umask() sets the calling process's file mode creation mask (umask) to
/// mask & 0777 (i.e., only the file permission bits of mask are used),
/// and returns the previous value of the mask.
/// # WARNING
/// In current implementation, umask is always 0. This syscall won't do anything.
pub fn sys_umask(mask: u32) -> isize {
    info!("[sys_umask] mask: {:o}", mask);
    warn!(
        "[sys_umask] In current implementation, umask is always 0. This syscall won't do anything."
    );
    0
}

bitflags! {
    pub struct FaccessatMode: u32 {
        const F_OK = 0;
        const R_OK = 4;
        const W_OK = 2;
        const X_OK = 1;
    }
    pub struct FaccessatFlags: u32 {
        const AT_SYMLINK_NOFOLLOW = 0x100;
        const AT_EACCESS = 0x200;
    }
}

pub fn sys_faccessat2(dirfd: usize, pathname: *const u8, mode: u32, flags: u32) -> isize {
    let token = current_user_token();
    let pathname = match translated_str(token, pathname) {
        Ok(path) => path,
        Err(errno) => return errno,
    };
    let mode = match FaccessatMode::from_bits(mode) {
        Some(mode) => mode,
        None => {
            warn!("[sys_faccessat2] unknown mode");
            return EINVAL;
        }
    };
    let flags = match FaccessatFlags::from_bits(flags) {
        Some(flags) => flags,
        None => {
            warn!("[sys_faccessat2] unknown flags");
            return EINVAL;
        }
    };

    info!(
        "[sys_faccessat2] dirfd: {}, pathname: {}, mode: {:?}, flags: {:?}",
        dirfd as isize, pathname, mode, flags
    );

    // Do not check user's authority, because user group is not implemented yet.
    // All existing files can be accessed.
    match __openat(dirfd, pathname.as_str()) {
        Ok(_) => SUCCESS,
        Err(errno) => errno,
    }
}

bitflags! {
    pub struct MsyncFlags: u32 {
        const MS_ASYNC      =   1;
        const MS_INVALIDATE =   2;
        const MS_SYNC       =   4;
    }
}

pub fn sys_msync(addr: usize, length: usize, flags: u32) -> isize {
    if !VirtAddr::from(addr).aligned() {
        return EINVAL;
    }
    let flags = match MsyncFlags::from_bits(flags) {
        Some(flags) => flags,
        None => return EINVAL,
    };
    let task = current_task().unwrap();
    if !task
        .vm
        .lock()
        .contains_valid_buffer(addr, length, MapPermission::empty())
    {
        return ENOMEM;
    }
    info!(
        "[sys_msync] addr: {:X}, length: {:X}, flags: {:?}",
        addr, flags, flags
    );
    SUCCESS
}

pub fn sys_ftruncate(fd: usize, length: isize) -> isize {
    let task = current_task().unwrap();
    let fd_table = task.files.lock();
    let file_descriptor = match fd_table.get_ref(fd) {
        Ok(file_descriptor) => file_descriptor,
        Err(errno) => return errno,
    };
    match file_descriptor.truncate_size(length) {
        Ok(()) => SUCCESS,
        Err(errno) => errno,
    }
}
