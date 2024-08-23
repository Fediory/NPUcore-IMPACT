use bitflags::bitflags;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};
use lwext4_sys::ext4::*;

pub const S_IFIFO: u32 = 4096;
pub const S_IFCHR: u32 = 8192;
pub const S_IFBLK: u32 = 24576;
pub const S_IFDIR: u32 = 16384;
pub const S_IFREG: u32 = 32768;
pub const S_IFLNK: u32 = 40960;
pub const S_IFSOCK: u32 = 49152;
bitflags! {
    pub struct DebugFlags: u32 {
        const BALLOC = DEBUG_BALLOC;
        const BCACHE = DEBUG_BCACHE;
        const BITMAP = DEBUG_BITMAP;
        const BLOCK_GROUP = DEBUG_BLOCK_GROUP;
        const BLOCKDEV = DEBUG_BLOCKDEV;
        const DIR_IDX = DEBUG_DIR_IDX;
        const DIR = DEBUG_DIR;
        const EXTENT = DEBUG_EXTENT;
        const FS = DEBUG_FS;
        const HASH = DEBUG_HASH;
        const IALLOC = DEBUG_IALLOC;
        const INODE = DEBUG_INODE;
        const SUPER = DEBUG_SUPER;
        const XATTR = DEBUG_XATTR;
        const MKFS = DEBUG_MKFS;
        const EXT4 = DEBUG_EXT4;
        const JBD = DEBUG_JBD;
        const MBR = DEBUG_MBR;
        const NOPREFIX = DEBUG_NOPREFIX;
        const ALL = DEBUG_ALL;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType {
    pub(super) mode: u32,
}

impl FileType {
    /// Create a new `FileType` from the given character.
    pub fn from_char(c: char) -> Self {
        match c {
            'b' => FileType { mode: S_IFBLK },
            'c' => FileType { mode: S_IFCHR },
            'd' => FileType { mode: S_IFDIR },
            'p' => FileType { mode: S_IFIFO },
            'l' => FileType { mode: S_IFLNK },
            's' => FileType { mode: S_IFSOCK },
            _ => FileType { mode: S_IFREG },
        }
    }
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.is(S_IFDIR)
    }
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.is(S_IFREG)
    }
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.is(S_IFLNK)
    }
    #[must_use]
    pub fn is_block_device(&self) -> bool {
        self.is(S_IFBLK)
    }
    #[must_use]
    pub fn is_char_device(&self) -> bool {
        self.is(S_IFCHR)
    }
    #[must_use]
    pub fn is_fifo(&self) -> bool {
        self.is(S_IFIFO)
    }
    #[must_use]
    pub fn is_socket(&self) -> bool {
        self.is(S_IFSOCK)
    }
    fn is(&self, ft: u32) -> bool {
        self.mode == ft
    }

    /// Get the character representing the file type.
    pub fn as_char(&self) -> char {
        match self.mode {
            S_IFBLK => 'b',
            S_IFCHR => 'c',
            S_IFDIR => 'd',
            S_IFIFO => 'p',
            S_IFLNK => 'l',
            S_IFREG => '-',
            S_IFSOCK => 's',
            _ => '?',
        }
    }
    pub(super) fn to_ext4(&self) -> u8 {
        (match self {
            FileType { mode: S_IFBLK } => EXT4_DE_BLKDEV,
            FileType { mode: S_IFCHR } => EXT4_DE_CHRDEV,
            FileType { mode: S_IFDIR } => EXT4_DE_DIR,
            FileType { mode: S_IFIFO } => EXT4_DE_FIFO,
            FileType { mode: S_IFLNK } => EXT4_DE_SYMLINK,
            FileType { mode: S_IFREG } => EXT4_DE_REG_FILE,
            FileType { mode: S_IFSOCK } => EXT4_DE_SOCK,
            _ => EXT4_DE_UNKNOWN,
        }) as u8
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MountStats(ext4_mount_stats);

impl MountStats {
    pub fn new() -> Self {
        Self(ext4_mount_stats {
            inodes_count: 0,
            free_inodes_count: 0,
            blocks_count: 0,
            free_blocks_count: 0,
            block_size: 0,
            block_group_count: 0,
            blocks_per_group: 0,
            inodes_per_group: 0,
            volume_name: [0; 16],
        })
    }
}

impl Debug for MountStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExtStats")
            .field("inodes_count", &self.inodes_count)
            .field("free_inodes_count", &self.free_inodes_count)
            .field("blocks_count", &self.blocks_count)
            .field("free_blocks_count", &self.free_blocks_count)
            .field("block_size", &self.block_size)
            .field("block_group_count", &self.block_group_count)
            .field("blocks_per_group", &self.blocks_per_group)
            .field("inodes_per_group", &self.inodes_per_group)
            .field("volume_name", &self.volume_name)
            .finish()
    }
}

impl Deref for MountStats {
    type Target = ext4_mount_stats;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MountStats {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

bitflags! {
    pub struct OpenFlags:u32{
        const RDONLY = O_RDONLY;
        const WRONLY = O_WRONLY;
        const RDWR = O_RDWR;
        const CREAT = O_CREAT;
        const EXCL = O_EXCL;
        const TRUNC = O_TRUNC;
        const APPEND = O_APPEND;
    }
}

#[derive(Clone)]
pub struct FileAttr {
    pub(super) raw: ext4_inode,
    pub(super) ino: u64,
}

impl FileAttr {
    pub fn empty() -> Self {
        Self {
            raw: ext4_inode {
                mode: 0,
                uid: 0,
                size_lo: 0,
                access_time: 0,
                change_inode_time: 0,
                modification_time: 0,
                deletion_time: 0,
                gid: 0,
                links_count: 0,
                blocks_count_lo: 0,
                flags: 0,
                unused_osd1: 0,
                blocks: [0; 15],
                generation: 0,
                file_acl_lo: 0,
                size_hi: 0,
                obso_faddr: 0,
                osd2: ext4_inode__bindgen_ty_1 {
                    linux2: ext4_inode__bindgen_ty_1__bindgen_ty_1 {
                        blocks_high: 0,
                        file_acl_high: 0,
                        uid_high: 0,
                        gid_high: 0,
                        checksum_lo: 0,
                        reserved2: 0,
                    },
                },
                extra_isize: 0,
                checksum_hi: 0,
                ctime_extra: 0,
                mtime_extra: 0,
                atime_extra: 0,
                crtime: 0,
                crtime_extra: 0,
                version_hi: 0,
            },
            ino: 0,
        }
    }
    fn uid(&self) -> u32 {
        unsafe { self.raw.uid as u32 | ((self.raw.osd2.linux2.uid_high as u32) << 16) }
    }
    fn gid(&self) -> u32 {
        unsafe { self.raw.gid as u32 | ((self.raw.osd2.linux2.gid_high as u32) << 16) }
    }
    fn size(&self) -> u64 {
        self.raw.size_lo as u64 | ((self.raw.size_hi as u64) << 32)
    }
    fn atime(&self) -> Time {
        Time::from_extra(self.raw.access_time, Some(self.raw.atime_extra))
    }
    fn mtime(&self) -> Time {
        Time::from_extra(self.raw.modification_time, Some(self.raw.mtime_extra))
    }
    fn ctime(&self) -> Time {
        Time::from_extra(self.raw.change_inode_time, Some(self.raw.ctime_extra))
    }
    fn crtime(&self) -> Time {
        Time::from_extra(self.raw.crtime, Some(self.raw.crtime_extra))
    }
    fn perm(&self) -> Permissions {
        Permissions(self.raw.mode as u32 & 0o777)
    }
    fn links(&self) -> u32 {
        self.raw.links_count as u32
    }
    fn file_type(&self) -> FileType {
        FileType {
            mode: ((self.raw.mode >> 12) << 12) as u32,
        }
    }
    fn st_ino(&self) -> u64 {
        self.ino
    }
    fn st_mode(&self) -> u32 {
        self.raw.mode as u32
    }
    fn st_atime(&self) -> i64 {
        self.atime().epoch_secs as i64
    }
    fn st_atime_nsec(&self) -> i64 {
        self.atime().nanos.map(|x| x as i64).unwrap_or(0)
    }
    fn st_mtime(&self) -> i64 {
        self.mtime().epoch_secs as i64
    }
    fn st_mtime_nsec(&self) -> i64 {
        self.mtime().nanos.map(|x| x as i64).unwrap_or(0)
    }
    fn st_ctime(&self) -> i64 {
        self.ctime().epoch_secs as i64
    }
    fn st_ctime_nsec(&self) -> i64 {
        self.ctime().nanos.map(|x| x as i64).unwrap_or(0)
    }
    fn st_blocks(&self) -> u64 {
        unsafe {
            self.raw.blocks_count_lo as u64 | ((self.raw.osd2.linux2.blocks_high as u64) << 32)
        }
    }
    fn st_rdev(&self) -> u32 {
        let blk = self.raw.blocks[0];
        let blk1 = self.raw.blocks[1];
        // if dev > 0xFFFF, blocks[1] is used
        // else blocks[0] is used
        blk | blk1
    }
}

impl Debug for FileAttr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileAttr")
            .field("uid", &self.uid())
            .field("gid", &self.gid())
            .field("size", &self.size())
            .field("atime", &self.atime())
            .field("mtime", &self.mtime())
            .field("ctime", &self.ctime())
            .field("crtime", &self.crtime())
            .field("perm", &self.perm())
            .field("links", &self.links())
            .finish()
    }
}

#[derive(Clone)]
pub struct Metadata(pub(super) FileAttr);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Permissions(pub(super) u32);

impl Permissions {
    /// Check if any class (owner, group, others) has write permission.
    pub fn readonly(&self) -> bool {
        // check if any class (owner, group, others) has write permission
        self.0 & 0o222 == 0
    }

    /// Set the readonly flag for all classes (owner, group, others).
    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            // remove write permission for all classes; equivalent to `chmod a-w <file>`
            self.0 &= !0o222;
        } else {
            // add write permission for all classes; equivalent to `chmod a+w <file>`
            self.0 |= 0o222;
        }
    }

    /// Create a new `Permissions` from the given mode bits.
    pub fn from_mode(mode: u32) -> Self {
        Self(mode & 0o777)
    }

    /// Get the mode bits for this set of permissions.
    pub fn mode(&self) -> u32 {
        self.0
    }

    /// Set the mode bits for this set of permissions.
    pub fn set_mode(&mut self, mode: u32) {
        *self = Self::from_mode(mode);
    }
}

impl Metadata {
    pub(super) fn set_ino(&mut self, ino: u64) {
        self.0.ino = ino;
    }

    #[must_use]
    pub fn file_type(&self) -> FileType {
        self.0.file_type()
    }

    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.file_type().is_dir()
    }

    #[must_use]
    pub fn is_file(&self) -> bool {
        self.file_type().is_file()
    }

    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.file_type().is_symlink()
    }

    #[must_use]
    pub fn len(&self) -> u64 {
        self.0.size()
    }

    #[must_use]
    pub fn permissions(&self) -> Permissions {
        self.0.perm()
    }

    pub fn modified(&self) -> Time {
        self.0.mtime()
    }

    pub fn accessed(&self) -> Time {
        self.0.atime()
    }

    pub fn created(&self) -> Time {
        self.0.ctime()
    }
}

impl Debug for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("file_type", &self.file_type())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("permissions", &self.permissions())
            .field("modified", &self.modified())
            .field("accessed", &self.accessed())
            .field("created", &self.created())
            .finish_non_exhaustive()
    }
}

pub trait MetaDataExt {
    fn ino(&self) -> u64;
    fn mode(&self) -> u32;
    fn nlink(&self) -> u64;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn size(&self) -> u64;
    fn atime(&self) -> i64;
    fn atime_nsec(&self) -> i64;
    fn mtime(&self) -> i64;
    fn mtime_nsec(&self) -> i64;
    fn ctime(&self) -> i64;
    fn ctime_nsec(&self) -> i64;
    fn blocks(&self) -> u64;
    fn rdev(&self) -> u32;
}

impl MetaDataExt for Metadata {
    fn ino(&self) -> u64 {
        self.0.st_ino()
    }
    fn mode(&self) -> u32 {
        self.0.st_mode()
    }
    fn nlink(&self) -> u64 {
        self.0.links() as u64
    }
    fn uid(&self) -> u32 {
        self.0.uid()
    }
    fn gid(&self) -> u32 {
        self.0.gid()
    }
    fn size(&self) -> u64 {
        self.0.size()
    }
    fn atime(&self) -> i64 {
        self.0.st_atime()
    }
    fn atime_nsec(&self) -> i64 {
        self.0.st_atime_nsec()
    }
    fn mtime(&self) -> i64 {
        self.0.st_mtime()
    }
    fn mtime_nsec(&self) -> i64 {
        self.0.st_mtime_nsec()
    }
    fn ctime(&self) -> i64 {
        self.0.st_ctime()
    }
    fn ctime_nsec(&self) -> i64 {
        self.0.st_ctime_nsec()
    }
    fn blocks(&self) -> u64 {
        self.0.st_blocks()
    }
    fn rdev(&self) -> u32 {
        self.0.st_rdev()
    }
}

/// A raw filesystem time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Time {
    pub epoch_secs: u64,
    pub nanos: Option<u32>,
}

impl Time {
    // c.f. ext4_decode_extra_time
    // "We use an encoding that preserves the times for extra epoch"
    // the lower two bits of the extra field are added to the top of the sec field,
    // the remainder are the nsec
    pub fn from_extra(epoch_secs: u32, extra: Option<u32>) -> Time {
        let mut epoch_secs = u64::from(epoch_secs);
        match extra {
            None => Time {
                epoch_secs,
                nanos: None,
            },
            Some(extra) => {
                let epoch_bits = 2;

                // 0b1100_00..0000
                let epoch_mask = (1 << epoch_bits) - 1;

                // 0b00..00_0011
                let nsec_mask = !0u32 << epoch_bits;

                epoch_secs += u64::from(extra & epoch_mask) << 32;

                let nanos = (extra & nsec_mask) >> epoch_bits;
                Time {
                    epoch_secs,
                    nanos: Some(nanos.clamp(0, 999_999_999)),
                }
            }
        }
    }
}

impl Into<u32> for Time {
    fn into(self) -> u32 {
        self.epoch_secs as u32
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {
    pub(super) accessed: Option<Time>,
    pub(super) modified: Option<Time>,
    pub(super) created: Option<Time>,
}

impl FileTimes {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_accessed(mut self, t: Time) -> Self {
        self.accessed = Some(t);
        self
    }

    pub fn set_modified(mut self, t: Time) -> Self {
        self.modified = Some(t);
        self
    }

    pub fn set_created(mut self, t: Time) -> Self {
        self.created = Some(t);
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FsType {
    Ext2 = 2,
    Ext3 = 3,
    Ext4 = 4,
}
