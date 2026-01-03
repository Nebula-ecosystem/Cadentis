use super::queue::TaskQueue;

use std::sync::Arc;

pub(crate) struct Executor {
    queue: Arc<TaskQueue>,
}

impl Executor {
    pub(crate) fn new(queue: Arc<TaskQueue>) -> Self {
        Self { queue }
    }

    pub fn run(&self) {
        while let Some(task) = self.queue.pop() {
            task.poll();
        }
    }
}
