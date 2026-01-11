use crate::runtime::task::Task;

use std::sync::Arc;
use std::task::Waker;

pub(crate) fn make_waker(task: Arc<Task>) -> Waker {}
