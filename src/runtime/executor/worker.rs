use crate::reactor::ReactorHandle;
use crate::runtime::context::enter_context;
use crate::runtime::work_stealing::injector::Injector;
use crate::runtime::work_stealing::queue::LocalQueue;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct Worker {
    id: usize,
    queue: LocalQueue,

    injector: Arc<Injector>,
}

impl Worker {
    pub(crate) fn new(id: usize, queue: LocalQueue, injector: Arc<Injector>) -> Self {
        Self {
            id,
            queue,
            injector,
        }
    }

    pub(crate) fn run(&self, shutdown: Arc<AtomicBool>, reactor: ReactorHandle) {
        loop {
            if shutdown.load(Ordering::Acquire) {
                break;
            }

            if let Some(task) = self.queue.pop() {
                enter_context(reactor.clone(), || {
                    task.run();
                });

                continue;
            }

            if let Some(task) = self.injector.steal() {
                enter_context(reactor.clone(), || {
                    task.run();
                });

                continue;
            }

            self.injector.park();
        }
    }
}
