use super::stream::TcpStream;
use crate::reactor::future::AcceptFuture;
use crate::reactor::poller::platform::{
    sys_bind, sys_close, sys_ipv6_is_necessary, sys_listen, sys_parse_sockaddr, sys_set_reuseaddr,
    sys_socket, sys_sockname,
};

use std::io;
use std::net::SocketAddr;
use std::os::fd::RawFd;

/// An asynchronous TCP listener.
///
/// `TcpListener` listens for incoming TCP connections and integrates
/// with the runtime reactor to accept connections without blocking.
///
/// It is the async equivalent of [`std::net::TcpListener`].
pub struct TcpListener {
    /// File descriptor of the listening socket.
    fd: RawFd,
}

impl TcpListener {
    /// Binds a TCP listener to the given address.
    ///
    /// The address must be a valid socket address string, such as
    /// `"127.0.0.1:8080"` or `"[::1]:8080"`.
    ///
    /// This function:
    /// - creates a non-blocking socket,
    /// - enables `SO_REUSEADDR`,
    /// - configures IPv6 dual-stack if applicable,
    /// - binds and starts listening.
    pub fn bind(address: &str) -> io::Result<Self> {
        let (storage, len) = sys_parse_sockaddr(address)?;
        let domain = storage.ss_family as i32;

        let fd = sys_socket(domain)?;

        sys_set_reuseaddr(fd)?;
        sys_ipv6_is_necessary(fd, domain)?;
        sys_bind(fd, &storage, len)?;
        sys_listen(fd)?;

        Ok(Self { fd })
    }

    /// Accepts an incoming TCP connection.
    ///
    /// This method asynchronously waits until a client connects,
    /// then returns a [`TcpStream`] and the peer address.
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (fd, address) = AcceptFuture::new(self.fd).await?;

        Ok((TcpStream::new(fd), address))
    }

    /// Returns the local socket address of this listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        sys_sockname(self.fd)
    }
}

impl Drop for TcpListener {
    /// Closes the listening socket.
    fn drop(&mut self) {
        sys_close(self.fd);
    }
}
