use crate::reactor::command::Command;
use crate::reactor::io::{IoEntry, Waiting};
use crate::reactor::poller::common::Interest;
use crate::reactor::poller::platform::{sys_read, sys_write};
use crate::runtime::context::CURRENT_REACTOR;

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct ReadFuture<'a> {
    fd: i32,
    buffer: &'a mut [u8],
    registered: bool,
}

impl<'a> ReadFuture<'a> {
    pub fn new(fd: i32, buffer: &'a mut [u8]) -> Self {
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
            return Poll::Ready(Ok(n as usize));
        }

        if n == 0 {
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

        Poll::Ready(Err(err))
    }
}

pub struct WriteFuture<'a> {
    fd: i32,
    buffer: &'a [u8],
    written: usize,
    registered: bool,
}

impl<'a> WriteFuture<'a> {
    pub fn new(fd: i32, buffer: &'a [u8]) -> Self {
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

            return Poll::Ready(Err(err));
        }

        Poll::Ready(Ok(this.written))
    }
}
