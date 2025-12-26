//! Global runtime context for thread-local task spawning.
//!
//! Provides a thread-local runtime handle that allows spawning tasks without
//! explicitly passing a runtime reference, similar to `tokio::spawn`.
//!
//! # Thread-Local Storage
//!
//! This module uses thread-local storage to maintain:
//! - The current runtime's task queue for spawning tasks
//! - The current runtime's reactor handle for I/O operations
//!
//! These are automatically set when entering a runtime context via [`Runtime::block_on`].
//!
//! [`Runtime::block_on`]: crate::runtime::Runtime::block_on

use super::queue::TaskQueue;
use crate::reactor::core::ReactorHandle;

use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// Thread-local storage for the current runtime's task queue.
    ///
    /// When a runtime is entered via [`Runtime::block_on`], its task queue is stored here,
    /// allowing [`Task::spawn`] to work without an explicit runtime reference.
    /// This enables global task spawning similar to `tokio::spawn()`.
    ///
    /// [`Runtime::block_on`]: crate::runtime::Runtime::block_on
    /// [`Task::spawn`]: crate::task::Task::spawn
    pub(crate) static CURRENT_QUEUE: RefCell<Option<Arc<TaskQueue>>> =
        const { RefCell::new(None) };

    /// Thread-local storage for the current runtime's reactor handle.
    ///
    /// When a runtime is entered via [`Runtime::block_on`], its reactor handle is stored here,
    /// allowing I/O operations to work without an explicit reactor reference.
    /// This enables patterns like [`TcpListener::bind`] without passing a reactor.
    ///
    /// [`Runtime::block_on`]: crate::runtime::Runtime::block_on
    /// [`TcpListener::bind`]: crate::net::tcp_listener::TcpListener::bind
    pub(crate) static CURRENT_REACTOR: RefCell<Option<ReactorHandle>> =
        const { RefCell::new(None) };
}

/// Enters a runtime context with the given task queue and reactor.
///
/// Sets the thread-local runtime context so that [`Task::spawn`] and I/O operations
/// can work without an explicit runtime handle.
///
/// # Arguments
/// * `queue` - The task queue to use as the current runtime context
/// * `reactor` - The reactor handle to use for I/O operations
/// * `function` - The function to execute within this runtime context
///
/// # Returns
/// The return value of the function `function`
///
/// [`Task::spawn`]: crate::task::Task::spawn
pub(crate) fn enter_context<F, R>(queue: Arc<TaskQueue>, reactor: ReactorHandle, function: F) -> R
where
    F: FnOnce() -> R,
{
    CURRENT_QUEUE.with(|current_queue| {
        CURRENT_REACTOR.with(|current_reactor| {
            let previous_queue = current_queue.borrow_mut().replace(queue.clone());
            let previous_reactor = current_reactor.borrow_mut().replace(reactor.clone());

            let result = function();

            *current_queue.borrow_mut() = previous_queue;
            *current_reactor.borrow_mut() = previous_reactor;

            result
        })
    })
}

/// Gets the current reactor handle from the thread-local context.
///
/// This is used internally by I/O operations like [`TcpListener::bind`]
/// to automatically use the current runtime's reactor without requiring
/// an explicit reactor parameter.
///
/// # Returns
/// The current reactor handle if inside a runtime context, or panics if called
/// outside of a runtime context (i.e., not within a [`Runtime::block_on`] call).
///
/// # Panics
/// Panics if called outside a runtime context. All I/O operations should be
/// performed within a [`Runtime::block_on`] call.
///
/// [`TcpListener::bind`]: crate::net::tcp_listener::TcpListener::bind
/// [`Runtime::block_on`]: crate::runtime::Runtime::block_on
pub fn current_reactor() -> ReactorHandle {
    CURRENT_REACTOR.with(|current| {
        current.borrow().clone().expect(
            "No reactor in current context. I/O operations must be called within Runtime::block_on",
        )
    })
}
