//! kqueue event wrapper and operations.
//!
//! This module wraps libc kevent structures and provides convenient methods for
//! registering and waiting for I/O and timer events using kqueue on macOS.
//!
//! # Event Types
//!
//! The module supports:
//! - Read events (EVFILT_READ): triggered when data is available to read
//! - Write events (EVFILT_WRITE): triggered when the socket is ready for writing
//! - Timer events (EVFILT_TIMER): triggered when a timer expires

use libc::{EV_ADD, EV_DELETE, EV_ENABLE, F_GETFL, F_SETFL, O_NONBLOCK, fcntl, kevent};
use std::ptr;

/// Wrapper around a kqueue event (kevent structure).
///
/// Provides a safe interface for working with kqueue events.
pub(crate) struct Event(kevent);

impl Event {
    /// An empty event constant used for array initialization.
    pub(crate) const EMPTY: Self = Self(kevent {
        ident: 0,
        filter: 0,
        flags: 0,
        fflags: 0,
        data: 0,
        udata: ptr::null_mut(),
    });

    /// Creates a new event for registration.
    ///
    /// # Arguments
    /// * `ident` - The identifier (typically a file descriptor or timer ID)
    /// * `filter` - The event filter type (EVFILT_READ, EVFILT_WRITE, EVFILT_TIMER)
    /// * `timer_milliseconds` - For timer events, the duration in milliseconds; None otherwise
    ///
    /// # Returns
    /// A new `Event` configured for registration with kqueue
    pub(crate) fn new(ident: usize, filter: i16, timer_milliseconds: Option<isize>) -> Self {
        let data = timer_milliseconds.unwrap_or(0);

        Self(kevent {
            ident,
            filter,
            flags: EV_ADD | EV_ENABLE,
            fflags: 0,
            data,
            udata: ptr::null_mut(),
        })
    }

    /// Gets the identifier of this event.
    ///
    /// # Returns
    /// The identifier (typically a file descriptor or timer ID)
    pub(crate) fn get_ident(&self) -> usize {
        self.0.ident
    }

    /// Gets the filter type of this event.
    ///
    /// # Returns
    /// The filter type (EVFILT_READ, EVFILT_WRITE, EVFILT_TIMER, etc.)
    pub(crate) fn get_filter(&self) -> i16 {
        self.0.filter
    }

    /// Registers this event with the kqueue.
    ///
    /// # Arguments
    /// * `queue` - The kqueue file descriptor
    pub(crate) fn register(&self, queue: i32) {
        unsafe {
            kevent(queue, &self.0, 1, ptr::null_mut(), 0, ptr::null());
        }
    }

    /// Unregisters an event from the kqueue.
    ///
    /// # Arguments
    /// * `queue` - The kqueue file descriptor
    /// * `ident` - The identifier to unregister
    /// * `filter` - The event filter type to unregister
    pub(crate) fn unregister(queue: i32, ident: usize, filter: i16) {
        let mut event = Self::new(ident, filter, None);
        event.0.flags = EV_DELETE;

        event.register(queue);
    }

    /// Waits for events to occur, blocking until at least one event is available.
    ///
    /// # Arguments
    /// * `queue` - The kqueue file descriptor
    /// * `events` - Buffer to store the retrieved events
    ///
    /// # Returns
    /// The number of events retrieved
    pub(crate) fn wait(queue: i32, events: &mut [Event; 64]) -> i32 {
        unsafe {
            kevent(
                queue,
                ptr::null(),
                0,
                events.as_mut_ptr() as *mut kevent,
                events.len() as i32,
                ptr::null(),
            )
        }
    }

    /// Polls for events without blocking.
    ///
    /// This is a non-blocking version of [`wait`](Self::wait). If no events are
    /// available, it returns immediately with 0.
    ///
    /// # Arguments
    /// * `queue` - The kqueue file descriptor
    /// * `events` - Buffer to store the retrieved events
    ///
    /// # Returns
    /// The number of events retrieved (0 if none available)
    pub(crate) fn try_wait(queue: i32, events: &mut [Event; 64]) -> i32 {
        let timespec = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        unsafe {
            kevent(
                queue,
                ptr::null(),
                0,
                events.as_mut_ptr() as *mut kevent,
                events.len() as i32,
                &timespec as *const libc::timespec,
            )
        }
    }

    /// Sets a file descriptor to non-blocking mode.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to configure
    pub(crate) fn set_nonblocking(file_descriptor: i32) {
        let flags = unsafe { fcntl(file_descriptor, F_GETFL) };

        unsafe {
            fcntl(file_descriptor, F_SETFL, flags | O_NONBLOCK);
        }
    }
}
