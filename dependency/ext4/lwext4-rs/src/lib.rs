#![feature(try_blocks)]
#![feature(error_in_core)]
#![cfg_attr(not(feature = "std"), no_std)]

mod block;
mod dir;
mod error;

#[cfg(feature = "std")]
mod standard;

extern crate alloc;
extern crate core;

#[cfg(feature = "std")]
pub use standard::*;

mod fs;

mod debug;
mod file;
mod mkfs;
mod types;

pub use block::{
    BlockDevice, BlockDeviceConfig, BlockDeviceInterface, MountHandle, RegisterHandle,
};
pub use debug::*;
pub use dir::{DirEntry, ReadDir};
pub use error::{Error, Result};
pub use file::File;
pub use fs::FileSystem;
pub use mkfs::{BuildExtFs, FsBuilder};
pub use types::{
    DebugFlags, FileTimes, FileType, FsType, MetaDataExt, Metadata, MountStats, Permissions, Time,
};
