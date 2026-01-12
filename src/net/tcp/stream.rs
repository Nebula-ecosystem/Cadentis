use crate::reactor::future::{ConnectFuture, ReadFuture, WriteFuture};
use crate::reactor::poller::platform::{
    sockaddr_storage_to_socketaddr, sys_parse_sockaddr, sys_set_reuseaddr, sys_socket,
};
use crate::reactor::poller::platform::{sys_close, sys_shutdown};

use std::io;
use std::net::Shutdown;
use std::os::fd::RawFd;

pub struct TcpStream {
    fd: RawFd,
}

impl TcpStream {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
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
                    "write returned zero bytes",
                ));
            }
            buffer = &buffer[n..];
        }

        Ok(())
    }

    pub async fn connect(address: &str) -> io::Result<Self> {
        let (storage, _) = sys_parse_sockaddr(address)?;
        let addr = sockaddr_storage_to_socketaddr(&storage)?;

        let fd = sys_socket(storage.ss_family)?;

        sys_set_reuseaddr(fd)?;
        ConnectFuture::new(fd, addr).await?;

        Ok(Self { fd })
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        sys_shutdown(self.fd, how)
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        sys_close(self.fd);
    }
}
