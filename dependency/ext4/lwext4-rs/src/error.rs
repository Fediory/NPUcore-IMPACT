use core::ffi::c_int as errno_t;
use core::fmt::{Display, Formatter};
use embedded_io::ErrorKind;
use lwext4_sys::ext4::*;

pub type Result<T> = core::result::Result<T, Error>;
/// from ext4_errno.h
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    OperationNotPermitted = EPERM as isize,
    NoEntry = ENOENT as isize,
    Io = EIO as isize,
    NoDeviceOrAddress = ENXIO as isize, //????
    TooBig = E2BIG as isize,
    OutOfMemory = ENOMEM as isize,
    PermissionDenied = EACCES as isize,
    BadAddress = EFAULT as isize,
    FileExists = EEXIST as isize,
    NoDevice = ENODEV as isize,
    NotDirectory = ENOTDIR as isize,
    IsDirectory = EISDIR as isize,
    InvalidArgument = EINVAL as isize,
    FileTooBig = EFBIG as isize,
    NoSpace = ENOSPC as isize,
    ReadOnly = EROFS as isize,
    TooManyLinks = EMLINK as isize,
    Range = ERANGE as isize,
    DirNotEmpty = ENOTEMPTY as isize,
    NoData = ENODATA as isize,
    NotSupported = ENOTSUP as isize,
    InvalidError = 9999,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl core::error::Error for Error {}

impl Error {
    fn from<T: Into<isize>>(value: T) -> Self {
        match value.into() as _ {
            EPERM => Error::OperationNotPermitted,
            ENOENT => Error::NoEntry,
            EIO => Error::Io,
            ENXIO => Error::NoDeviceOrAddress,
            E2BIG => Error::TooBig,
            ENOMEM => Error::OutOfMemory,
            EACCES => Error::PermissionDenied,
            EFAULT => Error::BadAddress,
            EEXIST => Error::FileExists,
            ENODEV => Error::NoDevice,
            ENOTDIR => Error::NotDirectory,
            EISDIR => Error::IsDirectory,
            EINVAL => Error::InvalidArgument,
            EFBIG => Error::FileTooBig,
            ENOSPC => Error::NoSpace,
            EROFS => Error::ReadOnly,
            EMLINK => Error::TooManyLinks,
            ERANGE => Error::Range,
            ENOTEMPTY => Error::DirNotEmpty,
            ENODATA => Error::NoData,
            ENOTSUP => Error::NotSupported,
            _ => Error::InvalidError,
        }
    }
}

pub fn result_to_errno(result: Result<()>) -> errno_t {
    (match result {
        Ok(()) => EOK,
        Err(e) => e as _,
    }) as errno_t
}

pub fn errno_to_result(errno: errno_t) -> Result<()> {
    if errno == EOK as i32 {
        Ok(())
    } else {
        Err(Error::from(errno as isize))
    }
}

impl From<Error> for core::fmt::Error {
    fn from(_value: Error) -> Self {
        core::fmt::Error
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        match value.raw_os_error() {
            Some(errno) => Error::from(errno as isize),
            None => Error::InvalidError,
        }
    }
}

#[cfg(feature = "std")]
impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        std::io::Error::from_raw_os_error(self as i32)
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::NoEntry => ErrorKind::NotFound,
            Error::Io => ErrorKind::Other,
            Error::NoDeviceOrAddress => ErrorKind::Other,
            Error::TooBig => ErrorKind::OutOfMemory,
            Error::OutOfMemory => ErrorKind::OutOfMemory,
            Error::PermissionDenied => ErrorKind::PermissionDenied,
            Error::BadAddress => ErrorKind::Other,
            Error::FileExists => ErrorKind::AlreadyExists,
            Error::NoDevice => ErrorKind::Other,
            Error::NotDirectory => ErrorKind::Other,
            Error::IsDirectory => ErrorKind::Other,
            Error::InvalidArgument => ErrorKind::InvalidInput,
            Error::FileTooBig => ErrorKind::Other,
            Error::NoSpace => ErrorKind::Other,
            Error::ReadOnly => ErrorKind::PermissionDenied,
            Error::TooManyLinks => ErrorKind::Other,
            Error::Range => ErrorKind::Other,
            Error::DirNotEmpty => ErrorKind::Other,
            Error::NoData => ErrorKind::Other,
            Error::NotSupported => ErrorKind::Other,
            Error::InvalidError => ErrorKind::Other,
            _ => ErrorKind::Other,
        }
    }
}
