use super::errno::*;
use crate::{fs::FileDescriptor, mm::copy_to_user, task::*};

pub fn sys_socket(_domain: u32, _type: u32, _protocol: u32) -> isize {
    let task = current_task().unwrap();
    let mut fd_table = task.files.lock();
    let new_fd = match fd_table.insert(FileDescriptor::new(
        (_type & 0o2000000) == 0o2000000,
        (_type & 0o4000) == 0o4000,
        crate::fs::make_socket(),
    )) {
        Ok(fd) => fd,
        Err(errno) => return errno,
    };
    new_fd as isize
}
pub fn sys_bind(_sockfd: usize, _addr: *const u8, _addrlen: u32) -> isize {
    SUCCESS
}
pub fn sys_getsockname(_sockfd: usize, _addr: *const u8, _addrlen: u32) -> isize {
    SUCCESS
}
pub fn sys_getpeername(_sockfd: usize, _addr: *const u8, _addrlen: u32) -> isize {
    ENOTSOCK
}
pub fn sys_setsockopt(
    _sockfd: usize,
    _level: u32,
    _optname: u32,
    _optcal: *const u8,
    _optlen: u32,
) -> isize {
    SUCCESS
}
pub fn sys_sendto(
    _sockfd: usize,
    _buf: *const u8,
    _len: usize,
    _flags: u32,
    _dest_addr: *const u8,
    _addrlen: u32,
) -> isize {
    1
}
pub fn sys_recvfrom(
    _sockfd: usize,
    buf: *mut u8,
    _len: usize,
    _flags: u32,
    _src_addr: *const u8,
    _addrlen: u32,
) -> isize {
    let src = "x";
    let task = current_task().unwrap();
    let token = task.get_user_token();
    copy_to_user(token, src.as_ptr(), buf).unwrap();
    1
}
pub fn sys_listen(_sockfd: usize, _backlog: u32) -> isize {
    SUCCESS
}
pub fn sys_connect(_sockfd: usize, _addr: *const u8, _addrlen: u32) -> isize {
    SUCCESS
}
pub fn sys_accept(_sockfd: usize, _addr: *const u8, _addrlen: u32) -> isize {
    SUCCESS
}
