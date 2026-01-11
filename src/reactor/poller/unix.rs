use std::os::fd::RawFd;

pub(crate) fn sys_read(fd: RawFd, buf: &mut [u8]) -> isize {
    unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) }
}

pub(crate) fn sys_write(fd: RawFd, buf: &mut [u8]) -> isize {
    unsafe { libc::write(fd, buf.as_mut_ptr() as *mut _, buf.len()) }
}

pub(crate) fn sys_close(fd: RawFd) {
    unsafe { libc::close(fd) };
}
