pub(crate) mod common;

#[cfg(target_os = "macos")]
mod kqueue;

#[cfg(target_os = "macos")]
pub(crate) type Poller = kqueue::KqueuePoller;

#[cfg(unix)]
pub(crate) mod unix;

#[cfg(unix)]
pub(crate) use unix as platform;
