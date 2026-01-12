use libc::{c_char, close, mkdir, mode_t, open, read, write};
use std::{ffi::c_uint, os::fd::RawFd};

pub(crate) fn sys_read(fd: RawFd, buffer: &mut [u8]) -> isize {
    unsafe { read(fd, buffer.as_mut_ptr() as *mut _, buffer.len()) }
}

pub(crate) fn sys_write(fd: RawFd, buffer: &[u8]) -> isize {
    unsafe { write(fd, buffer.as_ptr() as *mut _, buffer.len()) }
}

pub(crate) fn sys_close(fd: RawFd) {
    unsafe { close(fd) };
}

pub(crate) fn sys_open(path: *const c_char, flags: i32, mode: mode_t) -> RawFd {
    unsafe { open(path, flags, mode as c_uint) }
}

pub(crate) fn sys_mkdir(path: *const c_char, mode: mode_t) -> RawFd {
    unsafe { mkdir(path, mode) }
}
