use crate::reactor::ReactorHandle;
use crate::runtime::context::enter_context;
use crate::runtime::executor::worker::Worker;
use crate::runtime::task::Task;
use crate::runtime::work_stealing::injector::Injector;
use crate::runtime::work_stealing::queue::LocalQueue;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

pub(crate) struct Executor {
    injector: Arc<Injector>,
    handles: Vec<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl Executor {
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

    pub(crate) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.injector.notify_all();
    }

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

    pub(crate) fn join(&mut self) {
        for h in self.handles.drain(..) {
            let _ = h.join();
        }
    }
}
