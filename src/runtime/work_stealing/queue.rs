use crate::runtime::task::Task;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub(crate) struct LocalQueue {
    inner: Mutex<VecDeque<Arc<Task>>>,
}

impl LocalQueue {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn push(&self, task: Arc<Task>) {
        self.inner.lock().unwrap().push_back(task);
    }

    pub(crate) fn pop(&self) -> Option<Arc<Task>> {
        self.inner.lock().unwrap().pop_back()
    }

    pub(crate) fn steal(&self) -> Option<Arc<Task>> {
        self.inner.lock().unwrap().pop_front()
    }
}
