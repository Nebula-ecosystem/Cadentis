use crate::reactor::command::Command;
use crate::reactor::future::{ConnectFuture, ReadFutureStream, WriteFutureStream};
use crate::reactor::io::{IoEntry, Stream};
use crate::reactor::poller::common::Interest;
use crate::reactor::poller::platform::{
    sockaddr_storage_to_socketaddr, sys_close, sys_ipv6_is_necessary, sys_parse_sockaddr,
    sys_set_reuseaddr, sys_shutdown, sys_socket,
};
use crate::runtime::context::CURRENT_REACTOR;

use std::io;
use std::net::Shutdown;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};

/// An asynchronous TCP stream.
///
/// `TcpStream` is a non-blocking TCP connection integrated with the
/// runtime reactor. It supports buffered I/O and wakes tasks via the
/// reactor when the socket becomes readable or writable.
///
/// A `TcpStream` must be created and used **inside a running runtime**
/// (i.e. within a context where the reactor is available).
pub struct TcpStream {
    stream: Arc<Mutex<Stream>>,
}

impl TcpStream {
    /// Wraps an existing connected socket file descriptor.
    ///
    /// This method registers the socket with the reactor and enables
    /// readiness notifications for both read and write events.
    ///
    /// # Panics
    ///
    /// Panics if called outside of a running runtime (no reactor in context).
    pub fn new(fd: RawFd) -> Self {
        let stream = Arc::new(Mutex::new(Stream {
            fd,
            in_buffer: Vec::new(),
            out_buffer: Vec::new(),
            read_waiters: Vec::new(),
            write_waiters: Vec::new(),
        }));

        CURRENT_REACTOR.with(|cell| {
            let binding = cell.borrow();
            let reactor = binding.as_ref().expect("no reactor in context");

            let interest = Interest {
                read: true,
                write: true,
            };

            let _ = reactor.send(Command::Register {
                fd,
                interest,
                entry: IoEntry::Stream(stream.clone()),
            });
        });

        Self { stream }
    }

    /// Returns a future that reads up to `buffer.len()` bytes.
    ///
    /// This reads from the stream's internal input buffer filled by
    /// the reactor. If no data is available yet, the current task is
    /// registered as a read waiter.
    pub fn read<'a>(&'a self, buffer: &'a mut [u8]) -> ReadFutureStream<'a> {
        ReadFutureStream::new(self.stream.clone(), buffer)
    }

    /// Returns a future that writes data from `buffer`.
    ///
    /// The data is appended to the stream's output buffer and is flushed
    /// by the reactor when the socket becomes writable.
    pub fn write<'a>(&'a self, buffer: &'a [u8]) -> WriteFutureStream<'a> {
        WriteFutureStream::new(self.stream.clone(), buffer)
    }

    /// Writes the entire buffer to the stream.
    ///
    /// This method repeatedly calls [`write`](Self::write) until the
    /// full buffer has been queued and flushed.
    ///
    /// # Errors
    ///
    /// Returns `WriteZero` if the write operation reports progress of zero.
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

    /// Establishes a TCP connection to `address`.
    ///
    /// The address must be a string accepted by `SocketAddr::from_str`,
    /// e.g. `"127.0.0.1:8080"` or `"[::1]:8080"`.
    ///
    /// This creates a non-blocking socket, configures common options
    /// (such as `SO_REUSEADDR`), performs the connection, and then
    /// registers the stream with the reactor.
    pub async fn connect(address: &str) -> io::Result<Self> {
        let (storage, _) = sys_parse_sockaddr(address)?;
        let addr = sockaddr_storage_to_socketaddr(&storage)?;

        let domain = storage.ss_family as i32;
        let fd = sys_socket(domain)?;

        sys_set_reuseaddr(fd)?;
        sys_ipv6_is_necessary(fd, domain)?;

        ConnectFuture::new(fd, addr).await?;

        Ok(Self::new(fd))
    }

    /// Shuts down the read, write, or both halves of this connection.
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        sys_shutdown(self.stream.lock().unwrap().fd, how)
    }

    /// Splits the stream into a read half and a write half.
    ///
    /// Both halves share the underlying stream state and can be used
    /// concurrently.
    pub fn split(&self) -> (ReadHalf, WriteHalf) {
        (
            ReadHalf {
                stream: self.stream.clone(),
            },
            WriteHalf {
                stream: self.stream.clone(),
            },
        )
    }
}

impl Drop for TcpStream {
    /// Drops the stream.
    ///
    /// The underlying file descriptor is closed when the last reference
    /// to the shared stream state is dropped.
    fn drop(&mut self) {
        let fd = {
            let stream = self.stream.lock().unwrap();
            stream.fd
        };

        if Arc::strong_count(&self.stream) == 1 {
            sys_close(fd);
        }
    }
}

/// The read half of a [`TcpStream`], created by [`TcpStream::split`].
pub struct ReadHalf {
    stream: Arc<Mutex<Stream>>,
}

impl ReadHalf {
    /// Returns a future that reads up to `buffer.len()` bytes.
    pub fn read<'a>(&'a self, buffer: &'a mut [u8]) -> ReadFutureStream<'a> {
        ReadFutureStream::new(self.stream.clone(), buffer)
    }
}

/// The write half of a [`TcpStream`], created by [`TcpStream::split`].
pub struct WriteHalf {
    stream: Arc<Mutex<Stream>>,
}

impl WriteHalf {
    /// Returns a future that writes data from `buffer`.
    pub fn write<'a>(&'a self, buffer: &'a [u8]) -> WriteFutureStream<'a> {
        WriteFutureStream::new(self.stream.clone(), buffer)
    }

    /// Writes the entire buffer to the stream.
    ///
    /// This method repeatedly calls [`write`](Self::write) until the
    /// full buffer has been queued and flushed.
    ///
    /// # Errors
    ///
    /// Returns `WriteZero` if the write operation reports progress of zero.
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
}
