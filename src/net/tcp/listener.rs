use super::stream::TcpStream;
use crate::reactor::future::AcceptFuture;
use crate::reactor::poller::platform::{
    sys_bind, sys_close, sys_listen, sys_parse_sockaddr, sys_socket, sys_sockname,
};
use crate::reactor::poller::unix::sys_set_reuseaddr;

use std::io;
use std::net::SocketAddr;
use std::os::fd::RawFd;

pub struct TcpListener {
    fd: RawFd,
}

impl TcpListener {
    pub fn bind(address: &str) -> io::Result<Self> {
        let (storage, len) = sys_parse_sockaddr(address)?;
        let fd = sys_socket(storage.ss_family)?;

        sys_set_reuseaddr(fd)?;
        sys_bind(fd, &storage, len)?;
        sys_listen(fd)?;

        Ok(Self { fd })
    }

    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (fd, address) = AcceptFuture::new(self.fd).await?;

        Ok((TcpStream::new(fd), address))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        sys_sockname(self.fd)
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        sys_close(self.fd);
    }
}
