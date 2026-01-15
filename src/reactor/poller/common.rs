use std::os::fd::RawFd;

#[derive(Clone, Copy)]
pub(crate) struct Interest {
    pub(crate) read: bool,
    pub(crate) write: bool,
}

pub(crate) struct Waker(pub(crate) RawFd);

unsafe impl Send for Waker {}
unsafe impl Sync for Waker {}
