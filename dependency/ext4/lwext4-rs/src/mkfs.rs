use crate::alloc::string::ToString;
use crate::error::{errno_to_result, Result};
use crate::types::FsType;
use crate::{BlockDevice, BlockDeviceInterface, Error};
use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::CStr;
use core::fmt::Debug;
use core::mem::transmute;
use core::pin::Pin;
use core::ptr::null_mut;
use lwext4_sys::ext4::{ext4_fs, ext4_mkfs, ext4_mkfs_info, ext4_mkfs_read_info, ext4_sblock};
pub struct BuildExtFs<T: BlockDeviceInterface> {
    raw_fs: ext4_fs,
    device: Pin<Box<BlockDevice<T>>>,
    raw_info: ext4_mkfs_info,
}
#[derive(Debug, Clone)]
pub struct ExtFsInfo {
    pub len: u64,
    pub block_size: u32,
    pub blocks_per_group: u32,
    pub inodes_per_group: u32,
    pub inode_size: u32,
    pub inodes: u32,
    pub journal_blocks: u32,
    pub feat_ro_compat: u32,
    pub feat_compat: u32,
    pub feat_incompat: u32,
    pub bg_desc_reserve_blocks: u32,
    pub dsc_size: u16,
    pub uuid: [u8; 16usize],
    pub journal: bool,
    pub label: String,
}

impl From<ext4_mkfs_info> for ExtFsInfo {
    fn from(value: ext4_mkfs_info) -> Self {
        Self {
            len: value.len,
            block_size: value.block_size,
            blocks_per_group: value.blocks_per_group,
            inodes_per_group: value.inodes_per_group,
            inode_size: value.inode_size,
            inodes: value.inodes,
            journal_blocks: value.journal_blocks,
            feat_ro_compat: value.feat_ro_compat,
            feat_compat: value.feat_compat,
            feat_incompat: value.feat_incompat,
            bg_desc_reserve_blocks: value.bg_desc_reserve_blocks,
            dsc_size: value.dsc_size,
            uuid: value.uuid,
            journal: value.journal,
            label: unsafe { CStr::from_ptr(value.label) }
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl<T: BlockDeviceInterface> BuildExtFs<T> {
    fn new(bdev: Pin<Box<BlockDevice<T>>>, ext_fs_info: ext4_mkfs_info) -> Self {
        unsafe {
            Self {
                raw_fs: ext4_fs {
                    read_only: false,
                    bdev: transmute(&bdev.raw),
                    sb: ext4_sblock {
                        inodes_count: 0,
                        blocks_count_lo: 0,
                        reserved_blocks_count_lo: 0,
                        free_blocks_count_lo: 0,
                        free_inodes_count: 0,
                        first_data_block: 0,
                        log_block_size: 0,
                        log_cluster_size: 0,
                        blocks_per_group: 0,
                        frags_per_group: 0,
                        inodes_per_group: 0,
                        mount_time: 0,
                        write_time: 0,
                        mount_count: 0,
                        max_mount_count: 0,
                        magic: 0,
                        state: 0,
                        errors: 0,
                        minor_rev_level: 0,
                        last_check_time: 0,
                        check_interval: 0,
                        creator_os: 0,
                        rev_level: 0,
                        def_resuid: 0,
                        def_resgid: 0,
                        first_inode: 0,
                        inode_size: 0,
                        block_group_index: 0,
                        features_compatible: 0,
                        features_incompatible: 0,
                        features_read_only: 0,
                        uuid: [0; 16],
                        volume_name: [0; 16],
                        last_mounted: [0; 64],
                        algorithm_usage_bitmap: 0,
                        s_prealloc_blocks: 0,
                        s_prealloc_dir_blocks: 0,
                        s_reserved_gdt_blocks: 0,
                        journal_uuid: [0; 16],
                        journal_inode_number: 0,
                        journal_dev: 0,
                        last_orphan: 0,
                        hash_seed: [0; 4],
                        default_hash_version: 0,
                        journal_backup_type: 0,
                        desc_size: 0,
                        default_mount_opts: 0,
                        first_meta_bg: 0,
                        mkfs_time: 0,
                        journal_blocks: [0; 17],
                        blocks_count_hi: 0,
                        reserved_blocks_count_hi: 0,
                        free_blocks_count_hi: 0,
                        min_extra_isize: 0,
                        want_extra_isize: 0,
                        flags: 0,
                        raid_stride: 0,
                        mmp_interval: 0,
                        mmp_block: 0,
                        raid_stripe_width: 0,
                        log_groups_per_flex: 0,
                        checksum_type: 0,
                        reserved_pad: 0,
                        kbytes_written: 0,
                        snapshot_inum: 0,
                        snapshot_id: 0,
                        snapshot_r_blocks_count: 0,
                        snapshot_list: 0,
                        error_count: 0,
                        first_error_time: 0,
                        first_error_ino: 0,
                        first_error_block: 0,
                        first_error_func: [0; 32],
                        first_error_line: 0,
                        last_error_time: 0,
                        last_error_ino: 0,
                        last_error_line: 0,
                        last_error_block: 0,
                        last_error_func: [0; 32],
                        mount_opts: [0; 64],
                        usr_quota_inum: 0,
                        grp_quota_inum: 0,
                        overhead_clusters: 0,
                        backup_bgs: [0; 2],
                        encrypt_algos: [0; 4],
                        encrypt_pw_salt: [0; 16],
                        lpf_ino: 0,
                        padding: [0; 100],
                        checksum: 0,
                    },
                    inode_block_limits: [0; 4],
                    inode_blocks_per_level: [0; 4],
                    last_inode_bg_id: 0,
                    jbd_fs: null_mut(),
                    jbd_journal: null_mut(),
                    curr_trans: null_mut(),
                },
                device: bdev,
                raw_info: ext_fs_info,
            }
        }
    }
}
pub struct FsBuilder {
    block_size: u32,
    ty: Option<FsType>,
    journal: bool,
    label: Option<CString>,
}

impl FsBuilder {
    pub fn new() -> Self {
        Self {
            block_size: 1024,
            ty: None,
            journal: true,
            label: None,
        }
    }

    pub fn block_size(mut self, block_size: u32) -> Self {
        self.block_size = block_size;
        self
    }

    pub fn ty(mut self, ty: FsType) -> Self {
        self.ty = Some(ty);
        self
    }

    pub fn journal(mut self, journal: bool) -> Self {
        self.journal = journal;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(CString::new(label).unwrap());
        self
    }
    fn get_fs_info(&self) -> Result<ext4_mkfs_info> {
        match self.block_size {
            1024 | 2048 | 4096 => {}
            _ => return Err(Error::InvalidArgument),
        }
        let info = ext4_mkfs_info {
            len: 0,
            block_size: self.block_size,
            blocks_per_group: 0,
            inodes_per_group: 0,
            inode_size: 0,
            inodes: 0,
            journal_blocks: 0,
            feat_ro_compat: 0,
            feat_compat: 0,
            feat_incompat: 0,
            bg_desc_reserve_blocks: 0,
            dsc_size: 0,
            uuid: [0; 16],
            journal: self.journal,
            label: self
                .label
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(null_mut()),
        };
        Ok(info)
    }

    pub fn build<T: BlockDeviceInterface>(
        self,
        bdev: Pin<Box<BlockDevice<T>>>,
    ) -> Result<BuildExtFs<T>> {
        let ty = match self.ty {
            Some(ty) => ty,
            None => return Err(Error::InvalidArgument),
        };
        let info = self.get_fs_info()?;
        let mut fs = BuildExtFs::new(bdev, info);
        unsafe {
            errno_to_result(ext4_mkfs(
                &mut fs.raw_fs as _,
                transmute(&fs.device.raw),
                &mut fs.raw_info as _,
                ty as _,
            ))?;
        }
        Ok(fs)
    }
}

impl<T: BlockDeviceInterface> BuildExtFs<T> {
    pub fn take_device(self) -> Pin<Box<BlockDevice<T>>> {
        self.device
    }
    pub fn fs_info(&self) -> Result<ExtFsInfo> {
        unsafe {
            errno_to_result(ext4_mkfs_read_info(
                transmute(&self.device.raw),
                transmute(&self.raw_info),
            ))?;
        }
        Ok(self.raw_info.into())
    }
}
