use crate::task::{Task, state::COMPLETED};

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};

pub struct JoinHandle<T> {
    pub(crate) task: Arc<Task<T>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        if self.task.state.load(Ordering::Acquire) == COMPLETED {
            let value = unsafe {
                (*self.task.result.get())
                    .take()
                    .expect("result already taken")
            };
            return Poll::Ready(value);
        }

        self.task.waiters.lock().unwrap().push(cx.waker().clone());

        Poll::Pending
    }
}
