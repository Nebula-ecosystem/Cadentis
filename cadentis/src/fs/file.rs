use crate::reactor::future::{ReadFuture, WriteFuture};
use crate::reactor::poller::platform::{CREATEFLAGS, OPENFLAGS, sys_close, sys_open};

use std::ffi::CString;
use std::io;
use std::os::fd::RawFd;

pub struct File {
    fd: RawFd,
}

impl File {
    pub async fn open(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path)?;
        let fd = Self::open_with_flags(c_path, OPENFLAGS)?;

        Ok(Self { fd })
    }

    pub async fn create(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path)?;
        let fd = Self::open_with_flags(c_path, CREATEFLAGS)?;

        Ok(Self { fd })
    }

    fn open_with_flags(c_path: CString, flags: RawFd) -> io::Result<RawFd> {
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
