use crate::runtime::task::Runnable;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A per-worker local task queue.
///
/// `LocalQueue` stores runnable tasks local to a worker thread.
/// Tasks are normally pushed and popped from the back of the queue
/// (LIFO), which improves cache locality.
///
/// Other workers may steal tasks from the front of the queue (FIFO),
/// enabling work-stealing and load balancing across the executor.
pub(crate) struct LocalQueue {
    /// Inner deque protected by a mutex.
    inner: Mutex<VecDeque<Arc<dyn Runnable>>>,
}

impl LocalQueue {
    /// Creates an empty local task queue.
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    /// Pushes a runnable task onto the local queue.
    ///
    /// Tasks are pushed to the back of the queue.
    pub(crate) fn push(&self, task: Arc<dyn Runnable>) {
        self.inner.lock().unwrap().push_back(task);
    }

    /// Pops a runnable task from the local queue.
    ///
    /// Tasks are popped from the back of the queue.
    /// Returns `None` if the queue is empty.
    pub(crate) fn pop(&self) -> Option<Arc<dyn Runnable>> {
        self.inner.lock().unwrap().pop_back()
    }

    /// Steals a runnable task from the local queue.
    ///
    /// Stealing removes a task from the front of the queue and is
    /// intended to be used by other worker threads.
    ///
    /// Returns `None` if the queue is empty.
    pub(crate) fn steal(&self) -> Option<Arc<dyn Runnable>> {
        self.inner.lock().unwrap().pop_front()
    }
}
