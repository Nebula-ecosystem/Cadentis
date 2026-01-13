use crate::reactor::poller::common::Interest;

use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub(crate) enum IoEntry {
    Waiting(Waiting),
    Stream(Arc<Mutex<Stream>>),
}

impl IoEntry {
    pub(crate) fn wake_all(self) {
        match self {
            IoEntry::Waiting(waiting) => {
                waiting.waker.wake();
            }
            IoEntry::Stream(stream) => {
                let mut stream = stream.lock().unwrap();

                let read_waiters = std::mem::take(&mut stream.read_waiters);
                for w in read_waiters {
                    w.wake();
                }

                let write_waiters = std::mem::take(&mut stream.write_waiters);
                for w in write_waiters {
                    w.wake();
                }
            }
        }
    }
}

pub(crate) struct Waiting {
    pub(crate) waker: Waker,
    pub(crate) interest: Interest,
}

pub struct Stream {
    pub(crate) fd: RawFd,

    pub(crate) in_buffer: Vec<u8>,
    pub(crate) out_buffer: Vec<u8>,

    pub(crate) read_waiters: Vec<Waker>,
    pub(crate) write_waiters: Vec<Waker>,
}

impl Stream {
    pub(crate) fn interest(&self) -> Interest {
        Interest {
            read: true,
            write: true,
        }
    }
}
