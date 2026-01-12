use crate::reactor::future::{ReadFuture, WriteFuture};
use crate::reactor::poller::platform::sys_close;
use crate::reactor::poller::unix::sys_open;

use libc::{O_CREAT, O_NONBLOCK, O_RDONLY, O_TRUNC, O_WRONLY};
use std::ffi::CString;
use std::io;

pub struct File {
    fd: i32,
}

impl File {
    pub async fn open(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path)?;
        let flags = O_RDONLY | O_NONBLOCK;

        let fd = Self::open_with_flags(c_path, flags)?;

        Ok(Self { fd })
    }

    pub async fn create(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path)?;
        let flags = O_WRONLY | O_CREAT | O_TRUNC | O_NONBLOCK;

        let fd = Self::open_with_flags(c_path, flags)?;

        Ok(Self { fd })
    }

    fn open_with_flags(c_path: CString, flags: i32) -> io::Result<i32> {
        let fd = sys_open(c_path.as_ptr(), flags, 0o644);

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(fd)
    }

    pub fn read<'a>(&'a self, buffer: &'a mut [u8]) -> ReadFuture<'a> {
        ReadFuture::new(self.fd, buffer)
    }

    pub fn write<'a>(&'a self, buffer: &'a [u8]) -> WriteFuture<'a> {
        WriteFuture::new(self.fd, buffer)
    }

    pub async fn write_all(&self, mut buffer: &[u8]) -> io::Result<()> {
        while !buffer.is_empty() {
            let n = self.write(buffer).await?;

            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write entire buffer",
                ));
            }

            buffer = &buffer[n..];
        }

        Ok(())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        sys_close(self.fd);
    }
}
