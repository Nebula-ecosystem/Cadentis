//! Socket acceptance utilities for kqueue.
//!
//! This module provides helper functions for accepting new client connections
//! on a listening socket and registering them with the reactor.

use crate::reactor::core::Entry;
use crate::reactor::event::Event;
use crate::reactor::io::Connection;

use libc::{EAGAIN, EMFILE, ENFILE, EVFILT_READ, EWOULDBLOCK, accept};
use std::collections::HashMap;
use std::ptr;

/// Accepts a new client connection on the given listener socket.
///
/// This function attempts to accept a pending connection on the listener socket.
/// If successful, it configures the client socket as non-blocking and registers
/// it with the reactor for read events.
///
/// # Arguments
/// * `queue` - The kqueue file descriptor
/// * `registry` - The registry mapping file descriptors to entries
/// * `listener_file_descriptor` - The listening socket's file descriptor
///
/// # Behavior
/// - If no connection is pending (EAGAIN/EWOULDBLOCK), returns without error
/// - If the process has too many open files (EMFILE/ENFILE), returns without error
/// - Otherwise, accepts the connection and registers it for reading
pub(crate) fn accept_client(
    queue: i32,
    registry: &mut HashMap<i32, Entry>,
    listener_file_descriptor: i32,
) {
    let client_file_descriptor =
        unsafe { accept(listener_file_descriptor, ptr::null_mut(), ptr::null_mut()) };

    if client_file_descriptor < 0 {
        let error = get_errno();

        if error == EAGAIN || error == EWOULDBLOCK {
            return;
        }

        if error == EMFILE || error == ENFILE {
            return;
        }

        return;
    }

    Event::set_nonblocking(client_file_descriptor);

    let event = Event::new(client_file_descriptor as usize, EVFILT_READ, None);
    event.register(queue);

    registry.insert(client_file_descriptor, Entry::Client(Connection::new()));
}

/// Gets the last error number from the current thread.
///
/// # Returns
/// The errno value from the last system call that failed.
fn get_errno() -> i32 {
    unsafe { *libc::__error() }
}
