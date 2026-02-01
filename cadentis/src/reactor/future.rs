use crate::reactor::command::Command;
use crate::reactor::io::{IoEntry, Stream, Waiting};
use crate::reactor::poller::common::Interest;
use crate::reactor::poller::platform::{
    RawFd, sys_accept, sys_connect, sys_get_socket_error, sys_read, sys_write,
};
use crate::runtime::context::CURRENT_REACTOR;

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

/// Asynchronous read operation on a raw file descriptor.
///
/// This future attempts to read data into the provided buffer.
/// If the operation would block, it registers interest with the
/// reactor and yields until the file descriptor becomes readable.
///
/// The file descriptor **must** be in non-blocking mode.
pub struct ReadFuture<'a> {
    fd: RawFd,
    buffer: &'a mut [u8],
    registered: bool,
}

impl<'a> ReadFuture<'a> {
    /// Creates a new `ReadFuture`.
    pub fn new(fd: RawFd, buffer: &'a mut [u8]) -> Self {
        Self {
            fd,
            buffer,
            registered: false,
        }
    }
}

impl<'a> Future for ReadFuture<'a> {
    /// Returns the number of bytes read.
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let n = sys_read(this.fd, this.buffer);

        if n > 0 {
            deregister(this.fd, this.registered);
            return Poll::Ready(Ok(n as usize));
        }

        if n == 0 {
            deregister(this.fd, this.registered);
            return Poll::Ready(Ok(0));
        }

        let err = io::Error::last_os_error();

        if err.kind() == io::ErrorKind::WouldBlock {
            if !this.registered {
                CURRENT_REACTOR.with(|cell| {
                    let binding = cell.borrow();
                    let reactor = binding.as_ref().expect("no reactor in context");

                    let interest = Interest {
                        read: true,
                        write: false,
                    };

                    let _ = reactor.send(Command::Register {
                        fd: this.fd,
                        interest,
                        entry: IoEntry::Waiting(Waiting {
                            waker: cx.waker().clone(),
                            interest,
                        }),
                    });
                });

                this.registered = true;
            }

            return Poll::Pending;
        }

        deregister(this.fd, this.registered);
        Poll::Ready(Err(err))
    }
}

/// Asynchronous write operation on a raw file descriptor.
///
/// This future writes the entire buffer, yielding whenever the
/// operation would block. Partial writes are handled internally.
///
/// The file descriptor **must** be in non-blocking mode.
pub struct WriteFuture<'a> {
    fd: RawFd,
    buffer: &'a [u8],
    written: usize,
    registered: bool,
}

impl<'a> WriteFuture<'a> {
    /// Creates a new `WriteFuture`.
    pub fn new(fd: RawFd, buffer: &'a [u8]) -> Self {
        Self {
            fd,
            buffer,
            written: 0,
            registered: false,
        }
    }
}

impl<'a> Future for WriteFuture<'a> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        while this.written < this.buffer.len() {
            let n = sys_write(this.fd, &this.buffer[this.written..]);

            if n > 0 {
                this.written += n as usize;
                continue;
            }

            if n == 0 {
                deregister(this.fd, this.registered);
                return Poll::Ready(Ok(this.written));
            }

            let err = io::Error::last_os_error();

            if err.kind() == io::ErrorKind::WouldBlock {
                if !this.registered {
                    CURRENT_REACTOR.with(|cell| {
                        let binding = cell.borrow();
                        let reactor = binding.as_ref().expect("no reactor in context");

                        let interest = Interest {
                            read: false,
                            write: true,
                        };

                        let _ = reactor.send(Command::Register {
                            fd: this.fd,
                            interest,
                            entry: IoEntry::Waiting(Waiting {
                                waker: cx.waker().clone(),
                                interest,
                            }),
                        });
                    });

                    this.registered = true;
                }

                return Poll::Pending;
            }

            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }

            deregister(this.fd, this.registered);
            return Poll::Ready(Err(err));
        }

        deregister(this.fd, this.registered);
        Poll::Ready(Ok(this.written))
    }
}

/// Asynchronous accept operation on a listening socket.
///
/// Resolves with the newly accepted client file descriptor and
/// its peer address.
pub struct AcceptFuture {
    fd: RawFd,
    registered: bool,
}

impl AcceptFuture {
    /// Creates a new `AcceptFuture`.
    pub(crate) fn new(fd: RawFd) -> Self {
        Self {
            fd,
            registered: false,
        }
    }
}

impl Future for AcceptFuture {
    type Output = io::Result<(RawFd, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match sys_accept(this.fd) {
            Ok((client_fd, addr)) => {
                deregister(this.fd, this.registered);
                Poll::Ready(Ok((client_fd, addr)))
            }

            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                if !this.registered {
                    CURRENT_REACTOR.with(|cell| {
                        let binding = cell.borrow();
                        let reactor = binding.as_ref().expect("no reactor in context");

                        let interest = Interest {
                            read: true,
                            write: false,
                        };

                        let _ = reactor.send(Command::Register {
                            fd: this.fd,
                            interest,
                            entry: IoEntry::Waiting(Waiting {
                                waker: cx.waker().clone(),
                                interest,
                            }),
                        });
                    });

                    this.registered = true;
                }

                Poll::Pending
            }

            Err(err) => {
                deregister(this.fd, this.registered);
                Poll::Ready(Err(err))
            }
        }
    }
}

/// Asynchronous non-blocking connect operation.
pub struct ConnectFuture {
    fd: RawFd,
    addr: SocketAddr,
    started: bool,
    registered: bool,
}

impl ConnectFuture {
    /// Creates a new `ConnectFuture`.
    pub(crate) fn new(fd: RawFd, addr: SocketAddr) -> Self {
        Self {
            fd,
            addr,
            started: false,
            registered: false,
        }
    }
}

impl Future for ConnectFuture {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // If we already started the connection, check if it completed
        if this.started {
            match sys_get_socket_error(this.fd) {
                Ok(()) => {
                    deregister(this.fd, this.registered);
                    return Poll::Ready(Ok(()));
                }
                Err(err) => {
                    deregister(this.fd, this.registered);
                    return Poll::Ready(Err(err));
                }
            }
        }

        // First poll: initiate the connection
        match sys_connect(this.fd, &this.addr) {
            Ok(()) => {
                // Connected immediately (rare but possible on localhost)
                Poll::Ready(Ok(()))
            }

            Err(err)
                if err.kind() == io::ErrorKind::WouldBlock
                    || err.kind() == io::ErrorKind::InvalidInput  // EALREADY
                    || err.raw_os_error() == Some(libc::EINPROGRESS) =>
            {
                this.started = true;

                if !this.registered {
                    CURRENT_REACTOR.with(|cell| {
                        let binding = cell.borrow();
                        let reactor = binding.as_ref().expect("no reactor in context");

                        let interest = Interest {
                            read: false,
                            write: true,
                        };

                        let _ = reactor.send(Command::Register {
                            fd: this.fd,
                            interest,
                            entry: IoEntry::Waiting(Waiting {
                                waker: cx.waker().clone(),
                                interest,
                            }),
                        });
                    });

                    this.registered = true;
                }

                Poll::Pending
            }

            Err(err) => {
                deregister(this.fd, this.registered);
                Poll::Ready(Err(err))
            }
        }
    }
}

/// Deregisters an I/O interest from the reactor if it was previously registered.
fn deregister(fd: RawFd, registered: bool) {
    if registered {
        CURRENT_REACTOR.with(|cell| {
            if let Some(reactor) = cell.borrow().as_ref() {
                let _ = reactor.send(Command::Deregister { fd });
            }
        });
    }
}

/// Asynchronous read operation on a buffered stream.
///
/// Data is first read from the internal buffer filled by the reactor.
/// If no data is available, the task is registered as a read waiter.
pub struct ReadFutureStream<'a> {
    stream: Arc<Mutex<Stream>>,
    buffer: &'a mut [u8],
}

impl<'a> ReadFutureStream<'a> {
    /// Creates a new stream read future.
    pub fn new(stream: Arc<Mutex<Stream>>, buffer: &'a mut [u8]) -> Self {
        Self { stream, buffer }
    }
}

impl<'a> Future for ReadFutureStream<'a> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut stream = this.stream.lock().unwrap();

        if !stream.in_buffer.is_empty() {
            let n = std::cmp::min(this.buffer.len(), stream.in_buffer.len());

            this.buffer[..n].copy_from_slice(&stream.in_buffer[..n]);
            stream.in_buffer.drain(..n);

            return Poll::Ready(Ok(n));
        }

        stream.read_waiters.push(cx.waker().clone());

        if !stream.in_buffer.is_empty() {
            let n = std::cmp::min(this.buffer.len(), stream.in_buffer.len());

            this.buffer[..n].copy_from_slice(&stream.in_buffer[..n]);
            stream.in_buffer.drain(..n);

            return Poll::Ready(Ok(n));
        }

        Poll::Pending
    }
}

/// Asynchronous write operation on a buffered stream.
///
/// Data is appended to the stream output buffer and flushed by
/// the reactor when the file descriptor becomes writable.
pub struct WriteFutureStream<'a> {
    stream: Arc<Mutex<Stream>>,
    buffer: &'a [u8],
    written: usize,
}

impl<'a> WriteFutureStream<'a> {
    /// Creates a new stream write future.
    pub fn new(stream: Arc<Mutex<Stream>>, buffer: &'a [u8]) -> Self {
        Self {
            stream,
            buffer,
            written: 0,
        }
    }
}

impl<'a> Future for WriteFutureStream<'a> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut stream = this.stream.lock().unwrap();

        if this.written == 0 && !this.buffer.is_empty() {
            stream.out_buffer.extend_from_slice(this.buffer);
            this.written = this.buffer.len();
        }

        if stream.out_buffer.is_empty() {
            return Poll::Ready(Ok(this.written));
        }

        stream.write_waiters.push(cx.waker().clone());

        Poll::Pending
    }
}
