//! Platform-specific I/O poller abstraction.
//!
//! This module provides a unified interface over platform-specific
//! I/O polling mechanisms (such as kqueue on macOS).
//!
//! The poller is used by the reactor to:
//! - wait for I/O readiness events,
//! - wake the reactor when new commands arrive,
//! - integrate OS-level notifications with async tasks.
//!
//! The concrete implementation is selected at compile time
//! depending on the target operating system.

pub(crate) mod common;

pub(crate) use common::Waker;

#[cfg(target_os = "macos")]
mod kqueue;

#[cfg(target_os = "linux")]
mod epoll;

#[cfg(target_os = "macos")]
pub(crate) type Poller = kqueue::KqueuePoller;

#[cfg(target_os = "linux")]
pub(crate) type Poller = epoll::EpollPoller;

#[cfg(unix)]
pub(crate) mod unix;

#[cfg(unix)]
pub(crate) use unix as platform;
