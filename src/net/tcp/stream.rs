use crate::reactor::command::Command;
use crate::reactor::future::{ConnectFuture, ReadFutureStream, WriteFutureStream};
use crate::reactor::io::{IoEntry, Stream};
use crate::reactor::poller::platform::{
    sockaddr_storage_to_socketaddr, sys_close, sys_parse_sockaddr, sys_set_reuseaddr, sys_shutdown,
    sys_socket,
};
use crate::runtime::context::CURRENT_REACTOR;

use std::io;
use std::net::Shutdown;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};

pub struct TcpStream {
    stream: Arc<Mutex<Stream>>,
}

impl TcpStream {
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
            let interest = {
                let s = stream.lock().unwrap();
                s.interest()
            };

            let _ = reactor.send(Command::Register {
                fd,
                interest,
                entry: IoEntry::Stream(stream.clone()),
            });

            let _ = reactor.send(Command::Wake);
        });

        Self { stream }
    }

    pub fn read<'a>(&'a self, buffer: &'a mut [u8]) -> ReadFutureStream<'a> {
        ReadFutureStream::new(self.stream.clone(), buffer)
    }

    pub fn write<'a>(&'a self, buffer: &'a [u8]) -> WriteFutureStream<'a> {
        WriteFutureStream::new(self.stream.clone(), buffer)
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

        Ok(Self::new(fd))
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        sys_shutdown(self.stream.lock().unwrap().fd, how)
    }

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

pub struct ReadHalf {
    stream: Arc<Mutex<Stream>>,
}

impl ReadHalf {
    pub fn read<'a>(&'a self, buffer: &'a mut [u8]) -> ReadFutureStream<'a> {
        ReadFutureStream::new(self.stream.clone(), buffer)
    }
}

pub struct WriteHalf {
    stream: Arc<Mutex<Stream>>,
}

impl WriteHalf {
    pub fn write<'a>(&'a self, buffer: &'a [u8]) -> WriteFutureStream<'a> {
        WriteFutureStream::new(self.stream.clone(), buffer)
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
}
