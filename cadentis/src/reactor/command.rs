use super::io::IoEntry;
use super::poller::common::Interest;

use std::os::fd::RawFd;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::task::Waker;
use std::time::Instant;

pub(crate) enum Command {
    Register {
        fd: RawFd,
        entry: IoEntry,
        interest: Interest,
    },
    Deregister {
        fd: RawFd,
    },
    SetTimer {
        deadline: Instant,
        waker: Waker,
        cancelled: Arc<AtomicBool>,
    },
    Shutdown,
}
