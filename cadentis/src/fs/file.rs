use crate::reactor::future::{ReadFuture, WriteFuture};
use crate::reactor::poller::platform::{CREATEFLAGS, OPENFLAGS, sys_close, sys_open};

use std::ffi::CString;
use std::io;
use std::os::fd::RawFd;

/// An asynchronous file handle.
///
/// `File` provides non-blocking file I/O operations integrated with
/// the runtime reactor. It is the async equivalent of `std::fs::File`.
///
/// All read and write operations return futures that complete when
/// the underlying file descriptor becomes ready.
pub struct File {
    /// File descriptor associated with this file.
    fd: RawFd,
}

impl File {
    /// Opens a file in read-only mode.
    ///
    /// The file is opened with non-blocking flags and integrated
    /// with the runtime reactor.
    pub async fn open(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path)?;
        let fd = Self::open_with_flags(c_path, OPENFLAGS)?;

        Ok(Self { fd })
    }

    /// Creates a file for writing, truncating it if it already exists.
    ///
    /// The file is opened with non-blocking flags and integrated
    /// with the runtime reactor.
    pub async fn create(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path)?;
        let fd = Self::open_with_flags(c_path, CREATEFLAGS)?;

        Ok(Self { fd })
    }

    /// Opens a file using the provided raw flags.
    fn open_with_flags(c_path: CString, flags: RawFd) -> io::Result<RawFd> {
        let fd = sys_open(c_path.as_ptr(), flags, 0o644);

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(fd)
    }

    /// Returns a future that reads up to `buffer.len()` bytes.
    ///
    /// The current task is suspended until the file descriptor
    /// becomes readable.
    pub fn read<'a>(&'a self, buffer: &'a mut [u8]) -> ReadFuture<'a> {
        ReadFuture::new(self.fd, buffer)
    }

    /// Returns a future that writes data from `buffer`.
    ///
    /// The current task is suspended until the file descriptor
    /// becomes writable.
    pub fn write<'a>(&'a self, buffer: &'a [u8]) -> WriteFuture<'a> {
        WriteFuture::new(self.fd, buffer)
    }

    /// Writes the entire buffer to the file.
    ///
    /// This method repeatedly calls [`write`](Self::write) until the
    /// full buffer has been written.
    ///
    /// # Errors
    ///
    /// Returns `WriteZero` if the write operation makes no progress.
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
    /// Closes the file descriptor.
    fn drop(&mut self) {
        sys_close(self.fd);
    }
}
