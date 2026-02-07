use super::io::IoEntry;

use nucleus::io::RawFd;
use nucleus::poll::Interest;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::task::Waker;
use std::time::Instant;

/// Commands sent to the reactor thread.
///
/// `Command` is the communication protocol between the executor
/// (and async primitives) and the reactor. Commands are processed
/// sequentially by the reactor event loop.
pub(crate) enum Command {
    /// Registers a file descriptor with the reactor.
    ///
    /// The associated [`IoEntry`] determines how the reactor will
    /// wake tasks when the file descriptor becomes ready.
    Register {
        /// File descriptor to register.
        fd: RawFd,

        /// I/O entry associated with the file descriptor.
        entry: IoEntry,

        /// I/O interests to monitor (read/write).
        interest: Interest,
    },

    /// Deregisters a file descriptor from the reactor.
    ///
    /// After deregistration, no further events will be generated
    /// for the file descriptor.
    Deregister {
        /// File descriptor to deregister.
        fd: RawFd,
    },

    /// Schedules a timer to fire at a specific deadline.
    ///
    /// The provided waker is called once the deadline is reached,
    /// unless the timer is cancelled beforehand.
    SetTimer {
        /// Absolute deadline when the timer should fire.
        deadline: Instant,

        /// Waker to notify when the timer expires.
        waker: Waker,

        /// Cancellation flag shared with the sleep future.
        cancelled: Arc<AtomicBool>,
    },

    /// Shuts down the reactor.
    ///
    /// This causes the reactor event loop to exit.
    Shutdown,
}
