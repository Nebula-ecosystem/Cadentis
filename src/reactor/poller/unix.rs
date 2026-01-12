use libc::{close, read, write};
use std::os::fd::RawFd;

pub(crate) fn sys_read(fd: RawFd, buffer: &mut [u8]) -> isize {
    unsafe { read(fd, buffer.as_mut_ptr() as *mut _, buffer.len()) }
}

pub(crate) fn sys_write(fd: RawFd, buffer: &[u8]) -> isize {
    unsafe { write(fd, buffer.as_ptr() as *mut _, buffer.len()) }
}

pub(crate) fn sys_close(fd: RawFd) {
    unsafe { close(fd) };
}
