#![allow(unused)]

use num_enum::TryFromPrimitive;

/// Success
pub const SUCCESS: isize = 0;
/// Operation not permitted
pub const EPERM: isize = -1;
/// No such file or directory
pub const ENOENT: isize = -2;
/// No such process
pub const ESRCH: isize = -3;
/// Interrupted system call
pub const EINTR: isize = -4;
/// I/O error
pub const EIO: isize = -5;
/// No such device or address
pub const ENXIO: isize = -6;
/// Argument list too long
pub const E2BIG: isize = -7;
/// Exec format error
pub const ENOEXEC: isize = -8;
/// Bad file number
pub const EBADF: isize = -9;
/// No child processes
pub const ECHILD: isize = -10;
/// Try again
pub const EAGAIN: isize = -11;
/// Out of memory
pub const ENOMEM: isize = -12;
/// Permission denied
pub const EACCES: isize = -13;
/// Bad address
pub const EFAULT: isize = -14;
/// Block device required
pub const ENOTBLK: isize = -15;
/// Device or resource busy
pub const EBUSY: isize = -16;
/// File exists
pub const EEXIST: isize = -17;
/// Cross-device link
pub const EXDEV: isize = -18;
/// No such device
pub const ENODEV: isize = -19;
/// Not a directory
pub const ENOTDIR: isize = -20;
/// Is a directory
pub const EISDIR: isize = -21;
/// Invalid argument
pub const EINVAL: isize = -22;
/// File table overflow
pub const ENFILE: isize = -23;
/// Too many open files
pub const EMFILE: isize = -24;
/// Not a typewriter
pub const ENOTTY: isize = -25;
/// Text file busy
pub const ETXTBSY: isize = -26;
/// File too large
pub const EFBIG: isize = -27;
/// No space left on device
pub const ENOSPC: isize = -28;
/// Illegal seek
pub const ESPIPE: isize = -29;
/// Read-only file system
pub const EROFS: isize = -30;
/// Too many links
pub const EMLINK: isize = -31;
/// Broken pipe
pub const EPIPE: isize = -32;
/// Math argument out of domain of func
pub const EDOM: isize = -33;
/// Math result not representable
pub const ERANGE: isize = -34;
/// Resource deadlock would occur
pub const EDEADLK: isize = -35;
/// File name too long
pub const ENAMETOOLONG: isize = -36;
/// No record locks available
pub const ENOLCK: isize = -37;

/// Invalid system call number
/// # Note
/// This error code is special: arch syscall entry code will return
/// -ENOSYS if users try to call a syscall that doesn't exist.  To keep
/// failures of syscalls that really do exist distinguishable from
/// failures due to attempts to use a nonexistent syscall, syscall
/// implementations should refrain from returning -ENOSYS.
pub const ENOSYS: isize = -38;
/// Directory not empty
pub const ENOTEMPTY: isize = -39;
/// Too many symbolic links encountered
pub const ELOOP: isize = -40;
/// Operation would block
pub const EWOULDBLOCK: isize = EAGAIN;
/// No message of desired type
pub const ENOMSG: isize = -42;
/// Identifier removed
pub const EIDRM: isize = -43;
/// Channel number out of range
pub const ECHRNG: isize = -44;
/// Level 2 not synchronized
pub const EL2NSYNC: isize = -45;
/// Level 3 halted
pub const EL3HLT: isize = -46;
/// Level 3 reset
pub const EL3RST: isize = -47;
/// Link number out of range
pub const ELNRNG: isize = -48;
/// Protocol driver not attached
pub const EUNATCH: isize = -49;
/// No CSI structure available
pub const ENOCSI: isize = -50;
/// Level 2 halted
pub const EL2HLT: isize = -51;
/// Invalid exchange
pub const EBADE: isize = -52;
/// Invalid request descriptor
pub const EBADR: isize = -53;
/// Exchange full
pub const EXFULL: isize = -54;
/// No anode
pub const ENOANO: isize = -55;
/// Invalid request code
pub const EBADRQC: isize = -56;
/// Invalid slot
pub const EBADSLT: isize = -57;
/// Resource deadlock would occur
pub const EDEADLOCK: isize = EDEADLK;
/// Bad font file format
pub const EBFONT: isize = -59;
/// Device not a stream
pub const ENOSTR: isize = -60;
/// No data available
pub const ENODATA: isize = -61;
/// Timer expired
pub const ETIME: isize = -62;
/// Out of streams resources
pub const ENOSR: isize = -63;
/// Machine is not on the network
pub const ENONET: isize = -64;
/// Package not installed
pub const ENOPKG: isize = -65;
/// Object is remote
pub const EREMOTE: isize = -66;
/// Link has been severed
pub const ENOLINK: isize = -67;
/// Advertise error
pub const EADV: isize = -68;
/// Srmount error
pub const ESRMNT: isize = -69;
/// Communication error on send
pub const ECOMM: isize = -70;
/// Protocol error
pub const EPROTO: isize = -71;
/// Multihop attempted
pub const EMULTIHOP: isize = -72;
/// RFS specific error
pub const EDOTDOT: isize = -73;
/// Not a data message
pub const EBADMSG: isize = -74;
/// Value too large for defined data type
pub const EOVERFLOW: isize = -75;
/// Name not unique on network
pub const ENOTUNIQ: isize = -76;
/// File descriptor in bad state
pub const EBADFD: isize = -77;
/// Remote address changed
pub const EREMCHG: isize = -78;
/// Can not access a needed shared library
pub const ELIBACC: isize = -79;
/// Accessing a corrupted shared library
pub const ELIBBAD: isize = -80;
/// .lib section in a.out corrupted
pub const ELIBSCN: isize = -81;
/// Attempting to link in too many shared libraries
pub const ELIBMAX: isize = -82;
/// Cannot exec a shared library directly
pub const ELIBEXEC: isize = -83;
/// Illegal byte sequence
pub const EILSEQ: isize = -84;
/// Interrupted system call should be restarted
pub const ERESTART: isize = -85;
/// Streams pipe error
pub const ESTRPIPE: isize = -86;
/// Too many users
pub const EUSERS: isize = -87;
/// Socket operation on non-socket
pub const ENOTSOCK: isize = -88;
/// Destination address required
pub const EDESTADDRREQ: isize = -89;
/// Message too long
pub const EMSGSIZE: isize = -90;
/// Protocol wrong type for socket
pub const EPROTOTYPE: isize = -91;
/// Protocol not available
pub const ENOPROTOOPT: isize = -92;
/// Protocol not supported
pub const EPROTONOSUPPORT: isize = -93;
/// Socket type not supported
pub const ESOCKTNOSUPPORT: isize = -94;
/// Operation not supported on transport endpoint
pub const EOPNOTSUPP: isize = -95;
pub const ENOTSUP: isize = EOPNOTSUPP;
/// Protocol family not supported
pub const EPFNOSUPPORT: isize = -96;
/// Address family not supported by protocol
pub const EAFNOSUPPORT: isize = -97;
/// Address already in use
pub const EADDRINUSE: isize = -98;
/// Cannot assign requested address
pub const EADDRNOTAVAIL: isize = -99;
/// Network is down
pub const ENETDOWN: isize = -100;
/// Network is unreachable
pub const ENETUNREACH: isize = -101;
/// Network dropped connection because of reset
pub const ENETRESET: isize = -102;
/// Software caused connection abort
pub const ECONNABORTED: isize = -103;
/// Connection reset by peer
pub const ECONNRESET: isize = -104;
/// No buffer space available
pub const ENOBUFS: isize = -105;
/// Transport endpoint is already connected
pub const EISCONN: isize = -106;
/// Transport endpoint is not connected
pub const ENOTCONN: isize = -107;
/// Cannot send after transport endpoint shutdown
pub const ESHUTDOWN: isize = -108;
/// Too many references: cannot splice
pub const ETOOMANYREFS: isize = -109;
/// Connection timed out
pub const ETIMEDOUT: isize = -110;
/// Connection refused
pub const ECONNREFUSED: isize = -111;
/// Host is down
pub const EHOSTDOWN: isize = -112;
/// No route to host
pub const EHOSTUNREACH: isize = -113;
/// Operation already in progress
pub const EALREADY: isize = -114;
/// Operation now in progress
pub const EINPROGRESS: isize = -115;
/// Stale file handle
pub const ESTALE: isize = -116;
/// Structure needs cleaning
pub const EUCLEAN: isize = -117;
/// Not a XENIX named type file
pub const ENOTNAM: isize = -118;
/// No XENIX semaphores available
pub const ENAVAIL: isize = -119;
/// Is a named type file
pub const EISNAM: isize = -120;
/// Remote I/O error
pub const EREMOTEIO: isize = -121;
/// Quota exceeded
pub const EDQUOT: isize = -122;
/// No medium found
pub const ENOMEDIUM: isize = -123;
/// Wrong medium type
pub const EMEDIUMTYPE: isize = -124;
/// Operation Canceled
pub const ECANCELED: isize = -125;
/// Required key not available
pub const ENOKEY: isize = -126;
/// Key has expired
pub const EKEYEXPIRED: isize = -127;
/// Key has been revoked
pub const EKEYREVOKED: isize = -128;
/// Key was rejected by service
pub const EKEYREJECTED: isize = -129;
/// Owner died
/// (for robust mutexes)
pub const EOWNERDEAD: isize = -130;
/// State not recoverable
pub const ENOTRECOVERABLE: isize = -131;
/// Operation not possible due to RF-kill
pub const ERFKILL: isize = -132;
/// Memory page has hardware error
pub const EHWPOISON: isize = -133;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(isize)]
pub enum Errno {
    SUCCESS = 0,
    EPERM = -1,
    ENOENT = -2,
    ESRCH = -3,
    EINTR = -4,
    EIO = -5,
    ENXIO = -6,
    E2BIG = -7,
    ENOEXEC = -8,
    EBADF = -9,
    ECHILD = -10,
    EAGAIN = -11,
    ENOMEM = -12,
    EACCES = -13,
    EFAULT = -14,
    ENOTBLK = -15,
    EBUSY = -16,
    EEXIST = -17,
    EXDEV = -18,
    ENODEV = -19,
    ENOTDIR = -20,
    EISDIR = -21,
    EINVAL = -22,
    ENFILE = -23,
    EMFILE = -24,
    ENOTTY = -25,
    ETXTBSY = -26,
    EFBIG = -27,
    ENOSPC = -28,
    ESPIPE = -29,
    EROFS = -30,
    EMLINK = -31,
    EPIPE = -32,
    EDOM = -33,
    ERANGE = -34,
    EDEADLK = -35,
    ENAMETOOLONG = -36,
    ENOLCK = -37,
    ENOSYS = -38,
    ENOTEMPTY = -39,
    ELOOP = -40,
    ENOMSG = -42,
    EIDRM = -43,
    ECHRNG = -44,
    EL2NSYNC = -45,
    EL3HLT = -46,
    EL3RST = -47,
    ELNRNG = -48,
    EUNATCH = -49,
    ENOCSI = -50,
    EL2HLT = -51,
    EBADE = -52,
    EBADR = -53,
    EXFULL = -54,
    ENOANO = -55,
    EBADRQC = -56,
    EBADSLT = -57,
    EBFONT = -59,
    ENOSTR = -60,
    ENODATA = -61,
    ETIME = -62,
    ENOSR = -63,
    ENONET = -64,
    ENOPKG = -65,
    EREMOTE = -66,
    ENOLINK = -67,
    EADV = -68,
    ESRMNT = -69,
    ECOMM = -70,
    EPROTO = -71,
    EMULTIHOP = -72,
    EDOTDOT = -73,
    EBADMSG = -74,
    EOVERFLOW = -75,
    ENOTUNIQ = -76,
    EBADFD = -77,
    EREMCHG = -78,
    ELIBACC = -79,
    ELIBBAD = -80,
    ELIBSCN = -81,
    ELIBMAX = -82,
    ELIBEXEC = -83,
    EILSEQ = -84,
    ERESTART = -85,
    ESTRPIPE = -86,
    EUSERS = -87,
    ENOTSOCK = -88,
    EDESTADDRREQ = -89,
    EMSGSIZE = -90,
    EPROTOTYPE = -91,
    ENOPROTOOPT = -92,
    EPROTONOSUPPORT = -93,
    ESOCKTNOSUPPORT = -94,
    EOPNOTSUPP = -95,
    EPFNOSUPPORT = -96,
    EAFNOSUPPORT = -97,
    EADDRINUSE = -98,
    EADDRNOTAVAIL = -99,
    ENETDOWN = -100,
    ENETUNREACH = -101,
    ENETRESET = -102,
    ECONNABORTED = -103,
    ECONNRESET = -104,
    ENOBUFS = -105,
    EISCONN = -106,
    ENOTCONN = -107,
    ESHUTDOWN = -108,
    ETOOMANYREFS = -109,
    ETIMEDOUT = -110,
    ECONNREFUSED = -111,
    EHOSTDOWN = -112,
    EHOSTUNREACH = -113,
    EALREADY = -114,
    EINPROGRESS = -115,
    ESTALE = -116,
    EUCLEAN = -117,
    ENOTNAM = -118,
    ENAVAIL = -119,
    EISNAM = -120,
    EREMOTEIO = -121,
    EDQUOT = -122,
    ENOMEDIUM = -123,
    EMEDIUMTYPE = -124,
    ECANCELED = -125,
    ENOKEY = -126,
    EKEYEXPIRED = -127,
    EKEYREVOKED = -128,
    EKEYREJECTED = -129,
    EOWNERDEAD = -130,
    ENOTRECOVERABLE = -131,
    ERFKILL = -132,
    EHWPOISON = -133,
}

#[macro_export]
macro_rules! set_errno {
    ($errno:expr) => {};
}

#[macro_export]
macro_rules! errno_exit {
    ($errno:expr) => {
	set_errno!($errno)!;
        return expr;// or -1?
    };
}
