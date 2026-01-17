use crate::reactor::poller::common::Interest;
use crate::reactor::poller::platform::RawFd;

use std::sync::{Arc, Mutex};
use std::task::Waker;

/// An entry registered in the reactor for I/O readiness.
///
/// An `IoEntry` represents either:
/// - a one-shot waiter waiting for an I/O event, or
/// - a stream with potentially multiple read and write waiters.
///
/// This abstraction allows the reactor to treat simple waits and
/// stream-based I/O uniformly.
pub(crate) enum IoEntry {
    /// A single task waiting for an I/O event.
    Waiting(Waiting),

    /// A stream with internal read/write buffers and multiple waiters.
    Stream(Arc<Mutex<Stream>>),
}

impl IoEntry {
    /// Wakes all tasks associated with this I/O entry.
    ///
    /// - For [`Waiting`], wakes the single stored waker.
    /// - For [`Stream`], wakes all registered read and write waiters.
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

/// A single I/O wait registration.
///
/// Used for simple futures that wait for a specific I/O interest
/// (read or write) and only need to wake one task.
pub(crate) struct Waiting {
    /// Waker to notify when the I/O event occurs.
    pub(crate) waker: Waker,

    /// I/O interest being waited on.
    pub(crate) interest: Interest,
}

/// A stream registered with the reactor.
///
/// `Stream` represents a file descriptor with buffered I/O and
/// supports multiple concurrent readers and writers.
pub struct Stream {
    /// The underlying file descriptor.
    pub(crate) fd: RawFd,

    /// Input buffer used for read operations.
    pub(crate) in_buffer: Vec<u8>,

    /// Output buffer used for write operations.
    pub(crate) out_buffer: Vec<u8>,

    /// Tasks waiting for the stream to become readable.
    pub(crate) read_waiters: Vec<Waker>,

    /// Tasks waiting for the stream to become writable.
    pub(crate) write_waiters: Vec<Waker>,
}

impl Stream {
    /// Returns the I/O interests required for this stream.
    ///
    /// Streams are always interested in both read and write readiness.
    pub(crate) fn interest(&self) -> Interest {
        Interest {
            read: true,
            write: true,
        }
    }
}
