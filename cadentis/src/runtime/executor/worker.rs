use crate::reactor::ReactorHandle;
use crate::runtime::context::{CURRENT_WORKER_ID, enter_context};
use crate::runtime::work_stealing::injector::InjectorHandle;
use crate::runtime::work_stealing::queue::LocalQueue;
use crate::task::Runnable;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A worker thread in the executor.
///
/// A `Worker` is responsible for executing runnable tasks using
/// a work-stealing strategy. Each worker owns a local queue and
/// cooperates with other workers to balance load.
///
/// The execution order is:
/// 1. Pop from the local queue
/// 2. Steal from the global injector
/// 3. Steal from other workers
/// 4. Park if no work is available
pub(crate) struct Worker {
    /// Unique identifier of the worker.
    id: usize,

    /// All local queues (one per worker).
    ///
    /// Used for stealing work from other workers.
    locals: Arc<Vec<Arc<LocalQueue>>>,

    /// Handle to the global injector queue.
    injector: InjectorHandle,
}

impl Worker {
    /// Creates a new worker.
    ///
    /// # Arguments
    ///
    /// * `id` - Worker identifier
    /// * `locals` - Shared vector of all local queues
    /// * `injector` - Handle to the global injector
    pub(crate) fn new(
        id: usize,
        locals: Arc<Vec<Arc<LocalQueue>>>,
        injector: InjectorHandle,
    ) -> Self {
        Self {
            id,
            locals,
            injector,
        }
    }

    /// Runs the worker event loop.
    ///
    /// The worker repeatedly looks for work until a shutdown signal
    /// is received. While executing a task, the runtime context
    /// (reactor and injector) is installed for the current thread.
    ///
    /// # Execution loop
    ///
    /// - Execute from the local queue if possible
    /// - Otherwise, steal from the global injector
    /// - Otherwise, steal from another worker
    /// - Otherwise, park until work becomes available
    pub(crate) fn run(&self, shutdown: Arc<AtomicBool>, reactor: ReactorHandle) {
        CURRENT_WORKER_ID.with(|id| *id.borrow_mut() = Some(self.id));

        loop {
            if shutdown.load(Ordering::Acquire) {
                break;
            }

            if let Some(task) = self.locals[self.id].pop() {
                enter_context(reactor.clone(), self.injector.clone(), || {
                    task.run();
                });
                continue;
            }

            if let Some(task) = self.injector.steal() {
                enter_context(reactor.clone(), self.injector.clone(), || {
                    task.run();
                });
                continue;
            }

            if let Some(task) = self.try_steal() {
                enter_context(reactor.clone(), self.injector.clone(), || {
                    task.run();
                });
                continue;
            }

            self.injector.park();
        }
    }

    /// Attempts to steal a task from another worker's local queue.
    ///
    /// Workers are visited in a round-robin fashion to avoid
    /// starvation and distribute load evenly.
    fn try_steal(&self) -> Option<Arc<dyn Runnable>> {
        let len = self.locals.len();

        if len <= 1 {
            return None;
        }

        for i in 0..len {
            let victim = (self.id + i + 1) % len;

            if let Some(task) = self.locals[victim].steal() {
                return Some(task);
            }
        }
        None
    }
}
