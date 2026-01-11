use std::os::fd::RawFd;
use std::task::Waker;

use crate::reactor::poller::common::Interest;

pub(crate) enum IoEntry {
    Waiting(Waiting),
    Stream(Stream),
}

impl IoEntry {
    pub(crate) fn wake_all(self) {
        match self {
            IoEntry::Waiting(waiting) => {
                waiting.waker.wake();
            }
            IoEntry::Stream(mut stream) => {
                stream
                    .read_waiters
                    .drain(..)
                    .chain(stream.write_waiters.drain(..))
                    .for_each(|w| w.wake());
            }
        }
    }
}

pub(crate) struct Waiting {
    pub(crate) waker: Waker,
    pub(crate) interest: Interest,
}

pub(crate) struct Stream {
    pub(crate) fd: RawFd,

    pub(crate) in_buffer: Vec<u8>,
    pub(crate) out_buffer: Vec<u8>,

    pub(crate) read_waiters: Vec<Waker>,
    pub(crate) write_waiters: Vec<Waker>,
}

impl Stream {
    pub(crate) fn interest(&self) -> Interest {
        Interest {
            read: !self.read_waiters.is_empty(),
            write: !self.out_buffer.is_empty() || !self.write_waiters.is_empty(),
        }
    }
}
