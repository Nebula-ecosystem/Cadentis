use crate::task::Task;
use crate::task::state::COMPLETED;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};

/// A handle to a spawned task.
///
/// A `JoinHandle` allows awaiting the result of a task spawned onto
/// the runtime. It implements [`Future`] and resolves once the task
/// has completed.
///
/// Dropping the `JoinHandle` does **not** cancel the task; it only
/// discards the ability to observe its result.
pub struct JoinHandle<T> {
    /// Shared reference to the underlying task.
    pub(crate) task: Arc<Task<T>>,
}

impl<T> Future for JoinHandle<T> {
    /// The output of the spawned task.
    type Output = T;

    /// Polls the join handle.
    ///
    /// If the task has already completed, its result is returned
    /// immediately. Otherwise, the current waker is registered and
    /// the future returns `Poll::Pending`.
    ///
    /// The waker is registered **before** re-checking the task state
    /// to avoid missed wake-ups.
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

        if self.task.state.load(Ordering::Acquire) == COMPLETED {
            let value = unsafe {
                (*self.task.result.get())
                    .take()
                    .expect("result already taken")
            };

            return Poll::Ready(value);
        }

        Poll::Pending
    }
}
