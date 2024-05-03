use crate::error::{errno_to_result, result_to_errno, Error, Result};
use crate::types::MountStats;
use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_int as errno_t;
use core::ffi::c_void;
use core::intrinsics::transmute;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr::null_mut;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use log::info;
use lwext4_sys::ext4::*;

#[repr(transparent)]
pub struct BlockDevice<T: BlockDeviceInterface> {
    pub(super) raw: ext4_blockdev,
    data: PhantomData<T>,
}

impl<T: BlockDeviceInterface> BlockDevice<T> {
    pub fn new(interface: T) -> Pin<Box<BlockDevice<T>>> {
        unsafe {
            let raw_interface = ext4_blockdev_iface {
                open: Some(<T as BlockDeviceInterfaceExt>::open),
                bread: Some(<T as BlockDeviceInterfaceExt>::bread),
                bwrite: Some(<T as BlockDeviceInterfaceExt>::bwrite),
                close: Some(<T as BlockDeviceInterfaceExt>::close),
                lock: Some(<T as BlockDeviceInterfaceExt>::lock),
                unlock: Some(<T as BlockDeviceInterfaceExt>::unlock),
                ph_bsize: 0,
                ph_bcnt: 0,
                ph_bbuf: null_mut(),
                ph_refctr: 0,
                bread_ctr: 0,
                bwrite_ctr: 0,
                p_user: transmute(Box::leak(Box::new(interface))),
            };
            let device_raw = ext4_blockdev {
                bdif: Box::leak(Box::new(raw_interface)),
                part_offset: 0,
                part_size: 0,
                bc: null_mut(),
                lg_bsize: 0,
                lg_bcnt: 0,
                cache_write_back: 0,
                fs: null_mut(),
                journal: null_mut(),
            };
            Box::pin(Self {
                raw: device_raw,
                data: Default::default(),
            })
        }
    }
}

impl<T: BlockDeviceInterface> Drop for BlockDevice<T> {
    fn drop(&mut self) {
        info!("Dropping BlockDevice");
        unsafe {
            let block_size = (*self.raw.bdif).ph_bsize;
            let buf = (*self.raw.bdif).ph_bbuf;
            if !buf.is_null() {
                Vec::<u8>::from_raw_parts(buf, 0, block_size as usize);
            }
            let _ = Box::<T>::from_raw((*self.raw.bdif).p_user as _);
            let _ = Box::<ext4_blockdev_iface>::from_raw(self.raw.bdif as _);
        }
    }
}

impl<T: BlockDeviceInterface> Deref for BlockDevice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { transmute((*self.raw.bdif).p_user) }
    }
}
impl<T: BlockDeviceInterface> DerefMut for BlockDevice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute((*self.raw.bdif).p_user) }
    }
}

#[derive(Debug, Clone)]
pub struct CName(CString);

impl CName {
    pub fn new(name: String) -> Result<Self> {
        let c_name = CString::new(name).map_err(|_| Error::InvalidArgument)?;
        Ok(Self(c_name))
    }
    pub fn as_ptr(&self) -> *const i8 {
        self.0.as_ptr()
    }
    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap()
    }
    pub fn len(&self) -> usize {
        self.0.as_bytes_with_nul().len()
    }
}

pub struct RegisterHandle<T: BlockDeviceInterface> {
    #[allow(unused)]
    device: Pin<Box<BlockDevice<T>>>,
    dev_name: CName,
}

impl<T: BlockDeviceInterface> RegisterHandle<T> {
    /// Register a block device to the file system
    pub fn register(bdev: Pin<Box<BlockDevice<T>>>, dev_name: String) -> Result<Self> {
        let c_name = CName::new(dev_name)?;
        let handle = unsafe {
            errno_to_result(ext4_device_register(transmute(&bdev.raw), c_name.as_ptr()))?;
            RegisterHandle {
                device: bdev,
                dev_name: c_name,
            }
        };
        Ok(handle)
    }
    pub fn dev_name(&self) -> CName {
        self.dev_name.clone()
    }
}

impl<T: BlockDeviceInterface> Drop for RegisterHandle<T> {
    fn drop(&mut self) {
        info!("Unregistering {}", self.dev_name.as_str());
        unsafe {
            ext4_device_unregister(self.dev_name.as_ptr());
        }
    }
}

pub struct MountHandle<T: BlockDeviceInterface> {
    #[allow(unused)]
    register_handle: RegisterHandle<T>,
    pub(super) mount_point: CName,
}

impl<T: BlockDeviceInterface> MountHandle<T> {
    /// Mount a block device to the file system at the provided mount point
    pub fn mount(
        register_handle: RegisterHandle<T>,
        mount_point: String,
        journal_recovery: bool,
        read_only: bool,
    ) -> Result<Self> {
        let c_mount_point = CName::new(mount_point)?;
        let dev_name = register_handle.dev_name();
        unsafe {
            errno_to_result(ext4_mount(
                dev_name.as_ptr(),
                c_mount_point.as_ptr(),
                read_only,
            ))?;
            if journal_recovery {
                errno_to_result(ext4_recover(c_mount_point.as_ptr()))?;
            }
        };
        let handle = MountHandle {
            register_handle,
            mount_point: c_mount_point,
        };
        Ok(handle)
    }
    pub fn stats(&self) -> Result<MountStats> {
        let mut statfs = MountStats::new();
        unsafe {
            errno_to_result(ext4_mount_point_stats(
                self.mount_point.as_ptr(),
                statfs.deref_mut() as _,
            ))?;
        }
        Ok(statfs)
    }
}

impl<T: BlockDeviceInterface> Drop for MountHandle<T> {
    fn drop(&mut self) {
        info!("Unmounting {}", self.mount_point.as_str());
        unsafe {
            ext4_umount(self.mount_point.as_ptr());
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BlockDeviceConfig {
    pub block_size: u32,
    pub block_count: u64,
    pub part_size: u64,
    pub part_offset: u64,
}

pub trait BlockDeviceInterface {
    fn open(&mut self) -> Result<BlockDeviceConfig>;
    fn read_block(&mut self, buf: &mut [u8], block_id: u64, block_count: u32) -> Result<usize>;
    fn write_block(&mut self, buf: &[u8], block_id: u64, block_count: u32) -> Result<usize>;
    fn close(&mut self) -> Result<()>;
    fn lock(&mut self) -> Result<()>;
    fn unlock(&mut self) -> Result<()>;
}

trait BlockDeviceInterfaceExt {
    unsafe extern "C" fn open(bdev: *mut ext4_blockdev) -> errno_t;
    unsafe extern "C" fn bread(
        bdev: *mut ext4_blockdev,
        buf: *mut c_void,
        blk_id: u64,
        blk_cnt: u32,
    ) -> errno_t;
    unsafe extern "C" fn bwrite(
        bdev: *mut ext4_blockdev,
        buf: *const c_void,
        blk_id: u64,
        blk_cnt: u32,
    ) -> errno_t;
    unsafe extern "C" fn close(bdev: *mut ext4_blockdev) -> errno_t;
    unsafe extern "C" fn lock(bdev: *mut ext4_blockdev) -> errno_t;
    unsafe extern "C" fn unlock(bdev: *mut ext4_blockdev) -> errno_t;
}

impl<T: BlockDeviceInterface> BlockDeviceInterfaceExt for T {
    unsafe extern "C" fn open(bdev: *mut ext4_blockdev) -> errno_t {
        let r: Result<()> = try {
            let device: &mut BlockDevice<T> = transmute(bdev);
            let config = T::open(&mut *device)?;
            (*device.raw.bdif).ph_bsize = config.block_size;
            (*device.raw.bdif).ph_bcnt = config.block_count;
            (*device.raw.bdif).ph_bbuf = vec![0u8; config.block_size as usize].leak().as_mut_ptr();
            device.raw.part_size = config.part_size;
            device.raw.part_offset = config.part_offset;
        };
        result_to_errno(r)
    }

    unsafe extern "C" fn bread(
        bdev: *mut ext4_blockdev,
        buf: *mut c_void,
        blk_id: u64,
        blk_cnt: u32,
    ) -> errno_t {
        let r: Result<()> = try {
            let device: &mut BlockDevice<T> = transmute(bdev);
            let bsize = (*device.raw.bdif).ph_bsize;
            T::read_block(
                &mut *device,
                from_raw_parts_mut(transmute(buf), (blk_cnt * bsize) as usize),
                blk_id,
                blk_cnt,
            )?;
        };
        result_to_errno(r)
    }

    unsafe extern "C" fn bwrite(
        bdev: *mut ext4_blockdev,
        buf: *const c_void,
        blk_id: u64,
        blk_cnt: u32,
    ) -> errno_t {
        let r: Result<()> = try {
            let device: &mut BlockDevice<T> = transmute(bdev);
            let bsize = (*device.raw.bdif).ph_bsize;
            T::write_block(
                &mut *device,
                from_raw_parts(transmute(buf), (blk_cnt * bsize) as usize),
                blk_id,
                blk_cnt,
            )?;
        };
        result_to_errno(r)
    }

    unsafe extern "C" fn close(bdev: *mut ext4_blockdev) -> errno_t {
        let r: Result<()> = try {
            let device: &mut BlockDevice<T> = transmute(bdev);
            T::close(device)?;
        };
        result_to_errno(r)
    }

    unsafe extern "C" fn lock(bdev: *mut ext4_blockdev) -> errno_t {
        let r: Result<()> = try {
            let device: &mut BlockDevice<T> = transmute(bdev);
            T::lock(&mut *device)?;
        };
        result_to_errno(r)
    }

    unsafe extern "C" fn unlock(bdev: *mut ext4_blockdev) -> errno_t {
        let r: Result<()> = try {
            let device: &mut BlockDevice<T> = transmute(bdev);
            T::unlock(&mut *device)?;
        };
        result_to_errno(r)
    }
}
