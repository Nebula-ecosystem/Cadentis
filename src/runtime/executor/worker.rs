use crate::reactor::ReactorHandle;
use crate::runtime::context::{CURRENT_WORKER_ID, enter_context};
use crate::runtime::task::Task;
use crate::runtime::work_stealing::injector::InjectorHandle;
use crate::runtime::work_stealing::queue::LocalQueue;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct Worker {
    id: usize,
    locals: Arc<Vec<Arc<LocalQueue>>>,

    injector: InjectorHandle,
}

impl Worker {
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

    fn try_steal(&self) -> Option<Arc<Task>> {
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
