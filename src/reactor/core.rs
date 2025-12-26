//! Core reactor implementation for event-driven I/O.
//!
//! This module provides the [`Reactor`] type, which manages I/O events and timers
//! using kqueue on macOS. The reactor is responsible for:
//! - Registering file descriptors for read/write events
//! - Managing timers
//! - Polling for events
//! - Waking tasks when events occur
//!
//! # Architecture
//!
//! The reactor uses kqueue to efficiently wait for multiple I/O events and timers.
//! When an event occurs, the reactor wakes the associated task by calling its waker.
//!
//! # Thread Safety
//!
//! The reactor is wrapped in `Arc<Mutex<Reactor>>` to enable safe shared access
//! from multiple futures and threads.

use crate::reactor::event::Event;
use crate::reactor::io::{Connection, ConnectionState};
use crate::reactor::socket::accept_client;

use libc::{
    EAGAIN, EVFILT_READ, EVFILT_TIMER, EVFILT_WRITE, EWOULDBLOCK, close, kqueue, read, write,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::time::Duration;

/// A shared handle to a reactor, enabling multiple futures to access the same reactor.
///
/// This allows futures to register I/O and timer events with the reactor.
/// Using `Arc<Mutex<Reactor>>` enables thread-safe interior mutability and shared ownership
/// across multiple threads.
pub type ReactorHandle = Arc<Mutex<Reactor>>;

/// Represents an entry in the reactor's registry.
///
/// Each file descriptor in the reactor is associated with an entry that determines
/// how events for that file descriptor should be handled.
pub(crate) enum Entry {
    /// A listening socket that accepts new connections.
    #[allow(unused)]
    Listener,

    /// An active client connection.
    Client(Connection),

    /// A future waiting for an I/O event.
    Waiting(Waker),
}

/// The reactor manages I/O events and timers using kqueue.
///
/// The reactor is the core of the event loop. It maintains:
/// - A kqueue instance for polling events
/// - A registry of file descriptors and their associated entries
/// - A map of active timers
/// - A list of wakers to call when events occur
///
/// # Typical Usage
///
/// The reactor is typically wrapped in a `ReactorHandle` (Arc<Mutex<Reactor>>)
/// to allow shared access from multiple futures.
pub struct Reactor {
    /// The kqueue file descriptor used for event polling.
    queue: i32,

    /// Buffer for storing events returned by kqueue.
    events: [Event; 64],

    /// Number of events currently in the events buffer.
    n_events: i32,

    /// Registry mapping file descriptors to their entries.
    registry: HashMap<i32, Entry>,

    /// Map of timer IDs to their wakers.
    timers: HashMap<usize, Waker>,

    /// The next timer ID to assign.
    next_timer_id: usize,

    /// List of wakers to call when events are ready.
    wakers: Vec<Waker>,
}

// Reactor can be safely sent across threads because:
// - All fields are protected by the Mutex wrapper
// - The kqueue file descriptor is an integer (thread-safe)
// - HashMap<i32, Entry> accesses are synchronized by Mutex
// - Waker is Send/Sync
unsafe impl Send for Reactor {}
unsafe impl Sync for Reactor {}

const OUT_MAX_BYTES: usize = 8 * 1024 * 1024;

impl Reactor {
    /// Creates a new reactor instance.
    ///
    /// Initializes a new kqueue and prepares the reactor for event polling.
    ///
    /// # Returns
    /// A new `Reactor` with an empty registry and event buffer.
    pub(crate) fn new() -> Self {
        Self {
            queue: unsafe { kqueue() },
            events: [Event::EMPTY; 64],
            n_events: 0,
            registry: HashMap::new(),
            timers: HashMap::new(),
            next_timer_id: 1,
            wakers: Vec::new(),
        }
    }

    /// Registers a file descriptor for read events.
    ///
    /// When data becomes available to read on the file descriptor,
    /// the provided waker will be called.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to monitor
    /// * `waker` - The waker to call when the file descriptor is ready to read
    pub(crate) fn register_read(&mut self, file_descriptor: i32, waker: Waker) {
        let event = Event::new(file_descriptor as usize, EVFILT_READ, None);
        event.register(self.queue);

        self.registry.insert(file_descriptor, Entry::Waiting(waker));
    }

    /// Registers a file descriptor for write events.
    ///
    /// When the file descriptor becomes ready to write,
    /// the provided waker will be called.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to monitor
    /// * `waker` - The waker to call when the file descriptor is ready to write
    pub(crate) fn register_write(&mut self, file_descriptor: i32, waker: Waker) {
        let event = Event::new(file_descriptor as usize, EVFILT_WRITE, None);
        event.register(self.queue);

        self.registry.insert(file_descriptor, Entry::Waiting(waker));
    }

    /// Registers a timer that will fire after the specified duration.
    ///
    /// When the timer expires, the provided waker will be called.
    ///
    /// # Arguments
    /// * `duration` - How long to wait before firing the timer
    /// * `waker` - The waker to call when the timer expires
    pub(crate) fn register_timer(&mut self, duration: Duration, waker: Waker) {
        let milliseconds = duration.as_millis().clamp(0, isize::MAX as u128) as isize;
        let id = self.next_timer_id;
        self.next_timer_id = self.next_timer_id.wrapping_add(1).max(1);

        let event = Event::new(id, EVFILT_TIMER, Some(milliseconds));
        event.register(self.queue);

        self.timers.insert(id, waker);
    }

    /// Unregisters write events for a file descriptor.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to stop monitoring for writes
    fn unregister_write(&self, file_descriptor: i32) {
        Event::unregister(self.queue, file_descriptor as usize, EVFILT_WRITE);
    }

    /// Waits for events to occur, blocking until at least one event is available.
    ///
    /// This method will block the current thread until an event occurs.
    /// The events are stored in the internal buffer and can be processed
    /// by calling [`handle_events`](Self::handle_events).
    pub(crate) fn wait_for_event(&mut self) {
        let n_events = Event::wait(self.queue, &mut self.events);

        self.n_events = n_events;
    }

    /// Polls for I/O events without blocking and handles them if present.
    ///
    /// This is a non-blocking version of [`wait_for_event`](Self::wait_for_event).
    /// If events are available, they are stored and processed immediately.
    pub(crate) fn poll_events(&mut self) {
        let n_events = Event::try_wait(self.queue, &mut self.events);

        if n_events <= 0 {
            return;
        }

        self.n_events = n_events;
        self.handle_events();
    }

    /// Wakes all tasks that are ready to make progress.
    ///
    /// This method calls all wakers that were collected during event handling,
    /// allowing their associated tasks to be scheduled for execution.
    pub(crate) fn wake_ready(&mut self) {
        for waker in self.wakers.drain(..) {
            waker.wake();
        }
    }

    /// Processes all events that were retrieved by the last poll or wait.
    ///
    /// This method iterates through the events buffer and dispatches each event
    /// to the appropriate handler based on its type (read, write, or timer).
    ///
    /// # Event Types
    /// - **EVFILT_READ on Listener**: Accept a new client connection
    /// - **EVFILT_READ on Waiting**: Wake the waiting future
    /// - **EVFILT_READ on Client**: Read data from the client
    /// - **EVFILT_WRITE on Waiting**: Wake the waiting future
    /// - **EVFILT_WRITE on Client**: Write data to the client
    /// - **EVFILT_TIMER**: Fire the timer and wake the waiting future
    pub(crate) fn handle_events(&mut self) {
        for event in self.events.iter().take(self.n_events as usize) {
            let file_descriptor = event.get_ident() as i32;
            let filter = event.get_filter();

            match filter {
                // Handle read event on a listening socket
                EVFILT_READ
                    if matches!(self.registry.get(&(file_descriptor)), Some(Entry::Listener)) =>
                {
                    accept_client(self.queue, &mut self.registry, file_descriptor);
                }

                // Handle read event on other file descriptors
                EVFILT_READ => {
                    let mut entry = match self.registry.remove(&file_descriptor) {
                        Some(entry) => entry,
                        None => continue,
                    };

                    match &mut entry {
                        Entry::Waiting(waker) => {
                            // Preserve the waiting registration so subsequent readiness events
                            // continue to wake the associated future.
                            self.wakers.push(waker.clone());
                            self.registry.insert(file_descriptor, entry);
                            continue;
                        }
                        Entry::Client(connection)
                            if matches!(connection.state, ConnectionState::Reading) =>
                        {
                            let should_close = self.handle_read(file_descriptor, connection);

                            if should_close {
                                self.cleanup(file_descriptor);
                            } else {
                                self.registry.insert(file_descriptor, entry);
                            }
                        }
                        Entry::Client(_) => {
                            self.registry.insert(file_descriptor, entry);
                        }
                        _ => {
                            self.cleanup(file_descriptor);
                        }
                    }
                }

                // Handle write event
                EVFILT_WRITE => {
                    let mut entry = match self.registry.remove(&file_descriptor) {
                        Some(entry) => entry,
                        None => continue,
                    };

                    match &mut entry {
                        Entry::Waiting(waker) => {
                            // Preserve the waiting registration so subsequent readiness events
                            // continue to wake the associated future.
                            self.wakers.push(waker.clone());
                            self.registry.insert(file_descriptor, entry);
                            continue;
                        }
                        Entry::Client(connection)
                            if matches!(connection.state, ConnectionState::Writing) =>
                        {
                            let should_close = self.handle_write(file_descriptor, connection);

                            if should_close {
                                self.cleanup(file_descriptor);
                            } else {
                                self.registry.insert(file_descriptor, entry);
                            }
                        }
                        Entry::Client(_) => {
                            self.registry.insert(file_descriptor, entry);
                        }
                        _ => {
                            self.cleanup(file_descriptor);
                        }
                    }
                }

                // Handle timer event
                EVFILT_TIMER => {
                    let timer_id = event.get_ident();

                    if let Some(waker) = self.timers.remove(&timer_id) {
                        self.wakers.push(waker);
                    }
                }

                _ => {}
            }
        }
    }

    /// Handles a read event on a client connection.
    ///
    /// Attempts to read data from the client and buffer it for writing back.
    /// Transitions the connection from Reading to Writing state when data is read.
    ///
    /// # Arguments
    /// * `file_descriptor` - The client socket's file descriptor
    /// * `connection` - The connection state to update
    ///
    /// # Returns
    /// `true` if the connection should be closed, `false` otherwise
    ///
    /// # Close Conditions
    /// - The client closed the connection (read returns 0)
    /// - A non-recoverable error occurred
    /// - The output buffer would exceed the maximum size
    fn handle_read(&self, file_descriptor: i32, connection: &mut Connection) -> bool {
        let mut buffer = [0u8; 1024];
        let result = unsafe { read(file_descriptor, buffer.as_mut_ptr() as *mut _, buffer.len()) };

        // Connection closed by client
        if result == 0 {
            return true;
        }

        // Error occurred
        if result < 0 {
            let error = get_errno();

            // EAGAIN/EWOULDBLOCK means no data available right now
            if error == EAGAIN || error == EWOULDBLOCK {
                return false;
            }

            // Other errors mean we should close the connection
            return true;
        }

        let bytes_read = result as usize;

        // Check if adding this data would exceed the buffer limit
        if connection.out.len().saturating_add(bytes_read) > OUT_MAX_BYTES {
            return true;
        }

        // Buffer the data and transition to writing state
        connection.out.extend_from_slice(&buffer[..bytes_read]);
        connection.state = ConnectionState::Writing;

        false
    }

    /// Handles a write event on a client connection.
    ///
    /// Attempts to write buffered data to the client. When all data is written,
    /// transitions the connection back to Reading state.
    ///
    /// # Arguments
    /// * `file_descriptor` - The client socket's file descriptor
    /// * `connection` - The connection state to update
    ///
    /// # Returns
    /// `true` if the connection should be closed, `false` otherwise
    ///
    /// # Close Conditions
    /// - A non-recoverable error occurred
    fn handle_write(&self, file_descriptor: i32, connection: &mut Connection) -> bool {
        let result = unsafe {
            write(
                file_descriptor,
                connection.out.as_mut_ptr() as *mut _,
                connection.out.len(),
            )
        };

        // Error occurred
        if result < 0 {
            let error = get_errno();

            // EAGAIN/EWOULDBLOCK means socket not ready for writing right now
            if error == EAGAIN || error == EWOULDBLOCK {
                return false;
            }

            // Other errors mean we should close the connection
            return true;
        }

        // Remove the written bytes from the buffer
        let bytes_written = result as usize;
        connection.out.drain(..bytes_written);

        // If all data has been written, transition back to reading
        if connection.out.is_empty() {
            self.unregister_write(file_descriptor);
            connection.state = ConnectionState::Reading;
        }

        false
    }

    /// Cleans up a file descriptor by unregistering it and closing it.
    ///
    /// This method removes all event registrations for the file descriptor
    /// and closes the underlying socket.
    ///
    /// # Arguments
    /// * `file_descriptor` - The file descriptor to clean up
    fn cleanup(&self, file_descriptor: i32) {
        Event::unregister(self.queue, file_descriptor as usize, EVFILT_READ);
        Event::unregister(self.queue, file_descriptor as usize, EVFILT_WRITE);
        unsafe { close(file_descriptor) };
    }
}

/// Gets the last error number from the current thread.
///
/// # Returns
/// The errno value from the last system call that failed.
pub(crate) fn get_errno() -> i32 {
    unsafe { *libc::__error() }
}
