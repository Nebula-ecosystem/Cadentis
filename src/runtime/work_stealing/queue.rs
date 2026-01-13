use crate::runtime::task::Runnable;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub(crate) struct LocalQueue {
    inner: Mutex<VecDeque<Arc<dyn Runnable>>>,
}

impl LocalQueue {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn push(&self, task: Arc<dyn Runnable>) {
        self.inner.lock().unwrap().push_back(task);
    }

    pub(crate) fn pop(&self) -> Option<Arc<dyn Runnable>> {
        self.inner.lock().unwrap().pop_back()
    }

    pub(crate) fn steal(&self) -> Option<Arc<dyn Runnable>> {
        self.inner.lock().unwrap().pop_front()
    }
}
