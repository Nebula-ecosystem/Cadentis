use std::thread;

use super::Runtime;

pub struct RuntimeBuilder {
    worker_threads: usize,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        let worker_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self { worker_threads }
    }

    pub fn worker_threads(mut self, n: usize) -> Self {
        assert!(n > 0, "worker_threads must be > 0");

        self.worker_threads = n;
        self
    }

    pub fn build(self) -> Runtime {
        Runtime::new(self.worker_threads)
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
