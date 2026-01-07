use crate::core::task::Runnable;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub(crate) struct TaskQueue {
    pub(crate) queue: Mutex<VecDeque<Arc<dyn Runnable>>>,
}

impl TaskQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn push(&self, task: Arc<dyn Runnable>) {
        self.queue.lock().unwrap().push_back(task);
    }

    pub(crate) fn pop(&self) -> Option<Arc<dyn Runnable>> {
        self.queue.lock().unwrap().pop_front()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }
}
