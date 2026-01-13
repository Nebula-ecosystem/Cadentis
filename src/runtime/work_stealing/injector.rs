use crate::runtime::task::Runnable;

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub(crate) type InjectorHandle = Arc<Injector>;

pub(crate) struct Injector {
    queue: Mutex<VecDeque<Arc<dyn Runnable>>>,
    parked: Mutex<usize>,
    condvar: Condvar,
}

impl Injector {
    pub(crate) fn new() -> Self {
        Injector {
            queue: Mutex::new(VecDeque::new()),
            parked: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    pub(crate) fn notify_all(&self) {
        self.condvar.notify_all();
    }

    pub(crate) fn push(&self, task: Arc<dyn Runnable>) {
        self.queue.lock().unwrap().push_back(task);
        self.condvar.notify_one();
    }

    pub(crate) fn park(&self) {
        let mut parked = self.parked.lock().unwrap();
        *parked += 1;

        parked = self.condvar.wait(parked).unwrap();

        *parked -= 1;
    }

    pub(crate) fn steal(&self) -> Option<Arc<dyn Runnable>> {
        self.queue.lock().unwrap().pop_front()
    }
}
