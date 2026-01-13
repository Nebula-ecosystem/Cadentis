use crate::reactor::command::Command;
use crate::reactor::io::{IoEntry, Stream, Waiting};
use crate::reactor::poller::common::Interest;
use crate::reactor::poller::platform::{sys_read, sys_write};
use crate::reactor::poller::unix::{sys_accept, sys_connect};
use crate::runtime::context::CURRENT_REACTOR;

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::os::fd::RawFd;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

pub struct ReadFuture<'a> {
    fd: RawFd,
    buffer: &'a mut [u8],
    registered: bool,
}

impl<'a> ReadFuture<'a> {
    pub fn new(fd: RawFd, buffer: &'a mut [u8]) -> Self {
        Self {
            fd,
            buffer,
            registered: false,
        }
    }
}

impl<'a> Future for ReadFuture<'a> {
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

pub struct WriteFuture<'a> {
    fd: RawFd,
    buffer: &'a [u8],
    written: usize,
    registered: bool,
}

impl<'a> WriteFuture<'a> {
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

pub struct AcceptFuture {
    fd: RawFd,
    registered: bool,
}

impl AcceptFuture {
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

pub struct ConnectFuture {
    fd: RawFd,
    addr: SocketAddr,
    registered: bool,
}

impl ConnectFuture {
    pub(crate) fn new(fd: RawFd, addr: SocketAddr) -> Self {
        Self {
            fd,
            addr,
            registered: false,
        }
    }
}

impl Future for ConnectFuture {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match sys_connect(this.fd, &this.addr) {
            Ok(()) => {
                deregister(this.fd, this.registered);
                Poll::Ready(Ok(()))
            }

            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
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

fn deregister(fd: RawFd, registered: bool) {
    if registered {
        CURRENT_REACTOR.with(|cell| {
            if let Some(reactor) = cell.borrow().as_ref() {
                let _ = reactor.send(Command::Deregister { fd });
            }
        });
    }
}

pub struct ReadFutureStream<'a> {
    stream: Arc<Mutex<Stream>>,
    buffer: &'a mut [u8],
}

impl<'a> ReadFutureStream<'a> {
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

pub struct WriteFutureStream<'a> {
    stream: Arc<Mutex<Stream>>,
    buffer: &'a [u8],
    written: usize,
}

impl<'a> WriteFutureStream<'a> {
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
