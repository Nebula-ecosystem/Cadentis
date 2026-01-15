use crate::runtime::task::Runnable;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub(crate) type InjectorHandle = Arc<Injector>;

pub(crate) struct Injector {
    queue: Mutex<VecDeque<Arc<dyn Runnable>>>,
    parked: Mutex<usize>,
    condvar: Condvar,
    shutdown: AtomicBool,
}

impl Injector {
    pub(crate) fn new() -> Self {
        Injector {
            queue: Mutex::new(VecDeque::new()),
            parked: Mutex::new(0),
            condvar: Condvar::new(),
            shutdown: AtomicBool::new(false),
        }
    }

    pub(crate) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.condvar.notify_all();
    }

    pub(crate) fn push(&self, task: Arc<dyn Runnable>) {
        self.queue.lock().unwrap().push_back(task);
        self.condvar.notify_all();
    }

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

    pub(crate) fn steal(&self) -> Option<Arc<dyn Runnable>> {
        self.queue.lock().unwrap().pop_front()
    }
}
