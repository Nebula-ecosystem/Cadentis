use crate::runtime::task::Task;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

pub(crate) struct LocalQueue {
    inner: RefCell<VecDeque<Arc<Task>>>,
}

impl LocalQueue {
    pub(crate) fn new() -> Self {
        Self {
            inner: RefCell::new(VecDeque::new()),
        }
    }

    pub(crate) fn push(&self, task: Arc<Task>) {
        self.inner.borrow_mut().push_back(task);
    }

    pub(crate) fn pop(&self) -> Option<Arc<Task>> {
        self.inner.borrow_mut().pop_back()
    }
}
