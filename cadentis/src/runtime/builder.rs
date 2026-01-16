use super::Runtime;

use std::thread;

/// Builder for configuring and creating a runtime.
///
/// `RuntimeBuilder` allows customizing runtime parameters before
/// constructing the runtime. Currently, it supports configuring
/// the number of worker threads used by the executor.
///
/// # Examples
///
/// ```rust,ignore
/// let runtime = RuntimeBuilder::new()
///     .worker_threads(4)
///     .build();
/// ```
pub struct RuntimeBuilder {
    /// Number of worker threads in the executor.
    worker_threads: usize,
}

impl RuntimeBuilder {
    /// Creates a new `RuntimeBuilder` with default configuration.
    ///
    /// By default, the number of worker threads is set to the number
    /// of available logical CPUs, falling back to `1` if unavailable.
    pub fn new() -> Self {
        let worker_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self { worker_threads }
    }

    /// Sets the number of worker threads used by the runtime.
    ///
    /// # Panics
    ///
    /// Panics if `n == 0`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let builder = RuntimeBuilder::new()
    ///     .worker_threads(2);
    /// ```
    pub fn worker_threads(mut self, n: usize) -> Self {
        assert!(n > 0, "worker_threads must be > 0");

        self.worker_threads = n;
        self
    }

    /// Builds the runtime with the configured options.
    ///
    /// This starts the reactor and initializes the executor.
    pub fn build(self) -> Runtime {
        Runtime::new(self.worker_threads)
    }
}

impl Default for RuntimeBuilder {
    /// Creates a default `RuntimeBuilder`.
    fn default() -> Self {
        Self::new()
    }
}
