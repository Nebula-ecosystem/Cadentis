//! Async TCP futures for accepting connections and reading/writing data.
//!
//! This module provides low-level futures for TCP operations:
//! - [`AcceptFuture`]: Accepts a new client connection
//! - [`ReadFuture`]: Reads data from a file descriptor
//! - [`WriteFuture`]: Writes data to a file descriptor
//!
//! These futures integrate with the reactor to provide non-blocking I/O.

use crate::net::utils::sockaddr_to_socketaddr;
use crate::reactor::core::ReactorHandle;
use crate::reactor::event::Event;

use libc::{EAGAIN, EWOULDBLOCK, accept, read, sockaddr, sockaddr_in, socklen_t, write};

use std::future::Future;
use std::io;
use std::mem;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future that accepts a new client connection.
///
/// This future polls the listening socket until a client connection is available.
/// On the first poll that returns `EAGAIN`, it registers the socket with the reactor
/// to be woken when a connection becomes available.
pub struct AcceptFuture {
    /// The listening socket's file descriptor.
    listen_file_descriptor: i32,

    /// A handle to the reactor for registering read readiness.
    reactor: ReactorHandle,

    /// Whether we've registered with the reactor.
    registered: bool,
}

impl AcceptFuture {
    /// Creates a new accept future.
    ///
    /// # Arguments
    /// * `listen_file_descriptor` - The listening socket's file descriptor
    /// * `reactor` - A handle to the reactor
    pub fn new(listen_file_descriptor: i32, reactor: ReactorHandle) -> Self {
        Self {
            listen_file_descriptor,
            reactor,
            registered: false,
        }
    }
}

impl Future for AcceptFuture {
    type Output = io::Result<(i32, SocketAddr)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut addr: sockaddr_in = unsafe { mem::zeroed() };
        let mut addr_len: socklen_t = mem::size_of::<sockaddr_in>() as socklen_t;

        let client_file_descriptor = unsafe {
            accept(
                self.listen_file_descriptor,
                &mut addr as *mut _ as *mut sockaddr,
                &mut addr_len,
            )
        };

        // Connection accepted successfully
        if client_file_descriptor >= 0 {
            Event::set_nonblocking(client_file_descriptor);
            let socket_addr = sockaddr_to_socketaddr(&addr);

            return Poll::Ready(Ok((client_file_descriptor, socket_addr)));
        }

        let error = unsafe { *libc::__error() };

        // No connection available, register for wakeup
        if error == EAGAIN || error == EWOULDBLOCK {
            if !self.registered {
                self.reactor
                    .lock()
                    .unwrap()
                    .register_read(self.listen_file_descriptor, cx.waker().clone());
                self.registered = true;
            }

            return Poll::Pending;
        }

        // Other error occurred
        Poll::Ready(Err(io::Error::last_os_error()))
    }
}

/// A future that reads data from a file descriptor.
///
/// This future polls the file descriptor until data is available to read.
/// On the first poll that returns `EAGAIN`, it registers the file descriptor
/// with the reactor to be woken when data becomes available.
pub struct ReadFuture<'a> {
    /// The file descriptor to read from.
    file_descriptor: i32,

    /// The buffer to read data into.
    buffer: &'a mut [u8],

    /// A handle to the reactor for registering read readiness.
    reactor: ReactorHandle,

    /// Whether we've registered with the reactor.
    registered: bool,
}

impl<'a> ReadFuture<'a> {
    /// Creates a new read future.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to read from
    /// * `buffer` - The buffer to read data into
    /// * `reactor` - A handle to the reactor
    pub fn new(file_descriptor: i32, buffer: &'a mut [u8], reactor: ReactorHandle) -> Self {
        Self {
            file_descriptor,
            buffer,
            reactor,
            registered: false,
        }
    }
}

impl<'a> Future for ReadFuture<'a> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.as_mut().get_unchecked_mut() };

        let result = unsafe {
            read(
                this.file_descriptor,
                this.buffer.as_mut_ptr() as *mut _,
                this.buffer.len(),
            )
        };

        // Data was read successfully
        if result > 0 {
            return Poll::Ready(Ok(result as usize));
        }

        // Connection closed (EOF)
        if result == 0 {
            return Poll::Ready(Ok(0));
        }

        let error = unsafe { *libc::__error() };

        // No data available, register for wakeup
        if error == EAGAIN || error == EWOULDBLOCK {
            if !this.registered {
                this.reactor
                    .lock()
                    .unwrap()
                    .register_read(this.file_descriptor, cx.waker().clone());
                this.registered = true;
            }

            return Poll::Pending;
        }

        // Other error occurred
        Poll::Ready(Err(io::Error::last_os_error()))
    }
}

/// A future that writes data to a file descriptor.
///
/// This future polls the file descriptor until it's ready to accept data.
/// On the first poll that returns `EAGAIN`, it registers the file descriptor
/// with the reactor to be woken when the socket becomes writable.
pub struct WriteFuture<'a> {
    /// The file descriptor to write to.
    file_descriptor: i32,

    /// The buffer containing data to write.
    buffer: &'a [u8],

    /// A handle to the reactor for registering write readiness.
    reactor: ReactorHandle,

    /// Whether we've registered with the reactor.
    registered: bool,
}

impl<'a> WriteFuture<'a> {
    /// Creates a new write future.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to write to
    /// * `buffer` - The buffer containing data to write
    /// * `reactor` - A handle to the reactor
    pub fn new(file_descriptor: i32, buffer: &'a [u8], reactor: ReactorHandle) -> Self {
        Self {
            file_descriptor,
            buffer,
            reactor,
            registered: false,
        }
    }
}

impl<'a> Future for WriteFuture<'a> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.as_mut().get_unchecked_mut() };

        let result = unsafe {
            write(
                this.file_descriptor,
                this.buffer.as_ptr() as *const _,
                this.buffer.len(),
            )
        };

        // Data was written successfully
        if result >= 0 {
            return Poll::Ready(Ok(result as usize));
        }

        let error = unsafe { *libc::__error() };

        // Socket not ready for writing, register for wakeup
        if error == EAGAIN || error == EWOULDBLOCK {
            if !this.registered {
                this.reactor
                    .lock()
                    .unwrap()
                    .register_write(this.file_descriptor, cx.waker().clone());
                this.registered = true;
            }

            return Poll::Pending;
        }

        // Other error occurred
        Poll::Ready(Err(io::Error::last_os_error()))
    }
}
