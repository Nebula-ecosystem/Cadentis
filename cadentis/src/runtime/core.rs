use std::future::Future;
use std::sync::mpsc;

use super::executor::core::Executor;
use crate::reactor::command::Command;
use crate::reactor::{Reactor, ReactorHandle};

/// The main runtime handle.
///
/// `Runtime` is responsible for:
/// - spawning asynchronous tasks,
/// - driving task execution via the executor,
/// - integrating with the reactor for timers and I/O,
/// - providing a synchronous entry point via [`block_on`](Self::block_on).
///
/// Dropping the runtime shuts down all internal components in an orderly
/// fashion.
pub struct Runtime {
    /// Task executor responsible for scheduling and running futures.
    executor: Executor,

    /// Handle to the reactor thread.
    reactor_handle: ReactorHandle,
}

impl Runtime {
    /// Creates a new runtime instance.
    ///
    /// # Arguments
    ///
    /// * `worker_threads` - Number of worker threads used by the executor.
    ///
    /// The reactor is started automatically.
    pub(crate) fn new(worker_threads: usize) -> Self {
        let reactor_handle = Reactor::start();
        let executor = Executor::new(reactor_handle.clone(), worker_threads);

        Self {
            executor,
            reactor_handle,
        }
    }

    /// Spawns a future onto the runtime.
    ///
    /// The future is executed asynchronously and runs until completion.
    ///
    /// # Requirements
    ///
    /// - The future must be `Send`
    /// - The future must have `'static` lifetime
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// runtime.spawn(async {
    ///     // background task
    /// });
    /// ```
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.executor.spawn(future);
    }

    /// Runs a future to completion, blocking the current thread.
    ///
    /// This method is typically used as the synchronous entry point
    /// of the runtime (e.g. in `main` or tests).
    ///
    /// Internally, the future is spawned onto the executor and its
    /// result is sent back through a channel.
    ///
    /// # Panics
    ///
    /// Panics if the runtime shuts down before the future completes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let result = runtime.block_on(async {
    ///     42
    /// });
    /// assert_eq!(result, 42);
    /// ```
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (transmitter, receiver) = mpsc::channel();

        self.spawn(async move {
            let result = future.await;
            let _ = transmitter.send(result);
        });

        receiver.recv().expect("block_on failed")
    }
}

impl Drop for Runtime {
    /// Shuts down the runtime.
    ///
    /// This performs the following steps:
    /// 1. Stops task submission and signals the executor to shut down
    /// 2. Sends a shutdown command to the reactor
    /// 3. Joins all worker threads
    fn drop(&mut self) {
        self.executor.shutdown();

        let _ = self.reactor_handle.send(Command::Shutdown);

        self.executor.join();
    }
}
