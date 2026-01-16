use crate::reactor::ReactorHandle;
use crate::runtime::context::enter_context;
use crate::runtime::executor::worker::Worker;
use crate::runtime::task::Task;
use crate::runtime::work_stealing::injector::Injector;
use crate::runtime::work_stealing::queue::LocalQueue;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

/// Multi-threaded task executor.
///
/// The `Executor` is responsible for:
/// - spawning worker threads,
/// - coordinating task execution via work-stealing,
/// - integrating workers with the runtime context,
/// - managing orderly shutdown and thread joining.
///
/// It owns the global task injector and all worker threads.
pub(crate) struct Executor {
    /// Global injector queue shared by all workers.
    injector: Arc<Injector>,

    /// Join handles for worker threads.
    handles: Vec<JoinHandle<()>>,

    /// Shutdown flag shared with all workers.
    shutdown: Arc<AtomicBool>,
}

impl Executor {
    /// Creates a new executor with the given number of worker threads.
    ///
    /// This method:
    /// - initializes the global injector,
    /// - creates one local queue per worker,
    /// - spawns worker threads,
    /// - installs the runtime execution context for each worker.
    ///
    /// # Arguments
    ///
    /// * `reactor_handle` - Handle to the runtime reactor
    /// * `threads` - Number of worker threads
    pub(crate) fn new(reactor_handle: ReactorHandle, threads: usize) -> Self {
        let injector = Arc::new(Injector::new());
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut handles = Vec::with_capacity(threads);

        let mut locals = Vec::with_capacity(threads);
        for _ in 0..threads {
            locals.push(Arc::new(LocalQueue::new()));
        }

        let locals = Arc::new(locals);

        for id in 0..threads {
            let worker = Worker::new(id, locals.clone(), injector.clone());

            let reactor = reactor_handle.clone();
            let sd = shutdown.clone();
            let injector = injector.clone();

            let handle = thread::spawn(move || {
                enter_context(reactor.clone(), injector.clone(), || {
                    worker.run(sd, reactor);
                });
            });

            handles.push(handle);
        }

        Self {
            injector,
            handles,
            shutdown,
        }
    }

    /// Signals all workers to shut down.
    ///
    /// This method:
    /// - sets the shutdown flag,
    /// - wakes all parked workers via the injector.
    pub(crate) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.injector.shutdown();
    }

    /// Spawns a new asynchronous task onto the executor.
    ///
    /// Tasks spawned after shutdown has begun are silently ignored.
    pub(crate) fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.shutdown.load(Ordering::Acquire) {
            return;
        }

        let task = Arc::new(Task::new(future, self.injector.clone()));
        self.injector.push(task);
    }

    /// Waits for all worker threads to terminate.
    ///
    /// This should be called after initiating shutdown.
    pub(crate) fn join(&mut self) {
        for h in self.handles.drain(..) {
            let _ = h.join();
        }
    }
}
