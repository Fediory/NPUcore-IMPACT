use core::mem::size_of;

use crate::{arch::BLOCK_SZ, timer::TimeSpec};

bitflags! {
    pub struct OpenFlags: u32 {
        const O_RDONLY      =   0;
        const O_WRONLY      =   1;
        const O_RDWR        =   1 << 1;

        const O_CREAT       =   1 << 6;
        const O_EXCL        =   1 << 7;
        const O_NOCTTY      =   1 << 8;
        const O_TRUNC       =   1 << 9;

        const O_APPEND      =   1 << 10;
        const O_NONBLOCK    =   1 << 11;
        const O_DSYNC       =   1 << 12;
        const O_SYNC        =   1 << 12 | 1 << 20;
        const O_RSYNC       =   1 << 12 | 1 << 20;
        const O_ASYNC       =   1 << 13;
        const O_DIRECT      =   1 << 14;
        const O_LARGEFILE   =   1 << 15;
        const O_DIRECTORY   =   1 << 16;
        const O_NOFOLLOW    =   1 << 17;
        const O_NOATIME     =   1 << 18;
        const O_CLOEXEC     =   1 << 19;
        const O_PATH        =   1 << 21;
        const O_TMPFILE     =   1 << 16 | 1 << 22;
    }
}

bitflags! {
    pub struct SeekWhence: u32 {
        const SEEK_SET  =   0; /* set to offset bytes.  */
        const SEEK_CUR  =   1; /* set to its current location plus offset bytes.  */
        const SEEK_END  =   2; /* set to the size of the file plus offset bytes.  */
    }
}

bitflags! {
    pub struct StatMode: u32 {
        ///bit mask for the file type bit field
        const S_IFMT    =   0o170000;
        ///socket
        const S_IFSOCK  =   0o140000;
        ///symbolic link
        const S_IFLNK   =   0o120000;
        ///regular file
        const S_IFREG   =   0o100000;
        ///block device
        const S_IFBLK   =   0o060000;
        ///directory
        const S_IFDIR   =   0o040000;
        ///character device
        const S_IFCHR   =   0o020000;
        ///FIFO
        const S_IFIFO   =   0o010000;

        ///set-user-ID bit (see execve(2))
        const S_ISUID   =   0o4000;
        ///set-group-ID bit (see below)
        const S_ISGID   =   0o2000;
        ///sticky bit (see below)
        const S_ISVTX   =   0o1000;

        ///owner has read, write, and execute permission
        const S_IRWXU   =   0o0700;
        ///owner has read permission
        const S_IRUSR   =   0o0400;
        ///owner has write permission
        const S_IWUSR   =   0o0200;
        ///owner has execute permission
        const S_IXUSR   =   0o0100;

        ///group has read, write, and execute permission
        const S_IRWXG   =   0o0070;
        ///group has read permission
        const S_IRGRP   =   0o0040;
        ///group has write permission
        const S_IWGRP   =   0o0020;
        ///group has execute permission
        const S_IXGRP   =   0o0010;

        ///others (not in group) have read, write,and execute permission
        const S_IRWXO   =   0o0007;
        ///others have read permission
        const S_IROTH   =   0o0004;
        ///others have write permission
        const S_IWOTH   =   0o0002;
        ///others have execute permission
        const S_IXOTH   =   0o0001;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// Store the file attributes from a supported file.
pub struct Stat {
    /// ID of device containing file
    st_dev: u64,
    /// Inode number
    st_ino: u64,
    /// File type and mode   
    st_mode: u32,
    /// Number of hard links
    st_nlink: u32,
    /// User ID of the file's owner.
    st_uid: u32,
    /// Group ID of the file's group.
    st_gid: u32,
    /// Device ID (if special file)
    st_rdev: u64,
    __pad: u64,
    /// Size of file, in bytes.
    st_size: i64,
    /// Optimal block size for I/O.
    st_blksize: u32,
    __pad2: i32,
    /// Number 512-byte blocks allocated.
    st_blocks: u64,
    /// Backward compatibility. Used for time of last access.
    st_atime: TimeSpec,
    /// Time of last modification.
    st_mtime: TimeSpec,
    /// Time of last status change.
    st_ctime: TimeSpec,
    __unused: u64,
}

#[allow(unused)]
impl Stat {
    /// Get the inode number described in the `Stat`
    pub fn get_ino(&self) -> usize {
        self.st_ino as usize
    }
    pub fn get_size(&self) -> usize {
        self.st_size as usize
    }

    pub fn new(
        st_dev: u64,
        st_ino: u64,
        st_mode: u32,
        st_nlink: u32,
        st_rdev: u64,
        st_size: i64,
        st_atime_sec: i64,
        st_mtime_sec: i64,
        st_ctime_sec: i64,
    ) -> Self {
        const BLK_SIZE: u32 = BLOCK_SZ as u32;
        Self {
            st_dev,
            st_ino,
            st_mode,
            st_nlink,
            st_uid: 0,
            st_gid: 0,
            st_rdev,
            __pad: 0,
            st_size,
            st_blksize: BLK_SIZE as u32,
            __pad2: 0,
            st_blocks: (st_size as u64 + BLK_SIZE as u64 - 1) / BLK_SIZE as u64,
            st_atime: TimeSpec {
                tv_sec: st_atime_sec as usize,
                tv_nsec: 0,
            },
            st_mtime: TimeSpec {
                tv_sec: st_mtime_sec as usize,
                tv_nsec: 0,
            },
            st_ctime: TimeSpec {
                tv_sec: st_ctime_sec as usize,
                tv_nsec: 0,
            },
            __unused: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct StatxTimestamp {
    tv_sec: i64,
    tv_nsec: u32,
}

impl From<TimeSpec> for StatxTimestamp {
    fn from(value: TimeSpec) -> Self {
        Self {
            tv_sec: value.tv_sec as i64,
            tv_nsec: value.tv_nsec as u32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Statx {
    stx_mask: u32,
    stx_blksize: u32,
    stx_attributes: u64,
    stx_nlink: u32,
    stx_uid: u32,
    stx_gid: u32,
    stx_mode: u16,
    stx_ino: u64,
    stx_size: u64,
    stx_blocks: u64,
    stx_attributes_mask: u64,
    stx_atime: StatxTimestamp,
    stx_btime: StatxTimestamp,
    stx_ctime: StatxTimestamp,
    stx_mtime: StatxTimestamp,
    stx_rdev_major: u32,
    stx_rdev_minor: u32,
    stx_dev_major: u32,
    stx_dev_minor: u32,
    stx_mnt_id: u64,
    stx_dio_mem_align: u32,
    stx_dio_offset_align: u32,
}

impl Statx {
    pub fn new(
        stx_dev_major: u32,
        stx_dev_minor: u32,
        stx_ino: u64,
        stx_mode: u16,
        stx_nlink: u32,
        stx_rdev_major: u32,
        stx_rdev_minor: u32,
        stx_size: u64,
        stx_atime_sec: i64,
        stx_mtime_sec: i64,
        stx_ctime_sec: i64,
    ) -> Self {
        const BLK_SIZE: u32 = BLOCK_SZ as u32;
        Self {
            stx_mask: u32::MAX,     // TODO
            stx_attributes: 0,      // TODO
            stx_attributes_mask: 0, // TODO
            stx_dev_major,
            stx_dev_minor,
            stx_ino,
            stx_mode,
            stx_nlink,
            stx_uid: 0, // TODO
            stx_gid: 0, // TODO
            stx_rdev_major,
            stx_rdev_minor,
            stx_size,
            stx_blksize: BLK_SIZE as u32,
            stx_blocks: (stx_size as u64 + BLK_SIZE as u64 - 1) / BLK_SIZE as u64,
            stx_atime: StatxTimestamp {
                tv_sec: stx_atime_sec,
                tv_nsec: 0,
            },
            stx_btime: TimeSpec::new().into(), // TODO
            stx_mtime: StatxTimestamp {
                tv_sec: stx_mtime_sec,
                tv_nsec: 0,
            },
            stx_ctime: StatxTimestamp {
                tv_sec: stx_ctime_sec,
                tv_nsec: 0,
            },
            stx_mnt_id: 0,           // TODO
            stx_dio_mem_align: 0,    // TODO
            stx_dio_offset_align: 0, // TODO
        }
    }
}

const NAME_LIMIT: usize = 128;
#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// Native Linux directory entry structure.
/// # Note
/// In theory, the d_name may NOT have a fixed size and `d_name` may be arbitrarily lone.
pub struct Dirent {
    /// Inode number
    pub d_ino: usize,
    /// Offset to next `linux_dirent`
    pub d_off: isize,
    /// Length of this `linux_dirent`
    pub d_reclen: u16,
    /// Type of the file
    pub d_type: u8,
    /// The Filename (null-terminated)
    /// # Note
    /// We use fix-sized d_name array.
    pub d_name: [u8; NAME_LIMIT],
}

impl Dirent {
    /// Offset to next `linux_dirent`
    pub fn new(d_ino: usize, d_off: isize, d_type: u8, d_name: &str) -> Self {
        let mut dirent = Self {
            d_ino,
            d_off,
            d_reclen: size_of::<Self>() as u16,
            d_type,
            d_name: [0; NAME_LIMIT],
        };
        dirent.d_name[0..d_name.len()].copy_from_slice(d_name.as_bytes());
        dirent
    }
}
