use crate::runtime::task::Runnable;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Shared handle to the global task injector.
pub(crate) type InjectorHandle = Arc<Injector>;

/// Global task injector for the work-stealing scheduler.
///
/// The injector is used as a centralized queue where newly spawned
/// tasks are pushed before being picked up by worker threads.
///
/// It also coordinates worker parking and waking using a condition
/// variable, allowing workers to sleep when no work is available.
pub(crate) struct Injector {
    /// Queue holding globally injected tasks.
    queue: Mutex<VecDeque<Arc<dyn Runnable>>>,

    /// Number of parked worker threads.
    parked: Mutex<usize>,

    /// Condition variable used to wake parked workers.
    condvar: Condvar,

    /// Indicates whether the executor is shutting down.
    shutdown: AtomicBool,
}

impl Injector {
    /// Creates a new empty injector.
    pub(crate) fn new() -> Self {
        Injector {
            queue: Mutex::new(VecDeque::new()),
            parked: Mutex::new(0),
            condvar: Condvar::new(),
            shutdown: AtomicBool::new(false),
        }
    }

    /// Signals shutdown and wakes all parked workers.
    ///
    /// After shutdown is initiated, workers should stop parking
    /// and eventually exit.
    pub(crate) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.condvar.notify_all();
    }

    /// Pushes a new task into the global injector.
    ///
    /// This wakes any parked worker threads.
    pub(crate) fn push(&self, task: Arc<dyn Runnable>) {
        self.queue.lock().unwrap().push_back(task);
        self.condvar.notify_all();
    }

    /// Parks the current worker thread until work becomes available
    /// or a shutdown signal is received.
    ///
    /// Workers only park if the injector queue is empty.
    /// The park operation uses a timed wait to ensure periodic wakeups.
    pub(crate) fn park(&self) {
        if self.shutdown.load(Ordering::Acquire) {
            return;
        }

        if !self.queue.lock().unwrap().is_empty() {
            return;
        }

        let mut parked = self.parked.lock().unwrap();
        *parked += 1;

        let _ = self
            .condvar
            .wait_timeout(parked, Duration::from_millis(1))
            .unwrap();
    }

    /// Steals a task from the global injector.
    ///
    /// Tasks are taken from the front of the queue.
    /// Returns `None` if no tasks are available.
    pub(crate) fn steal(&self) -> Option<Arc<dyn Runnable>> {
        self.queue.lock().unwrap().pop_front()
    }
}
