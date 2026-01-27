use crate::task::Task;
use crate::task::set::SetHandle;
use crate::task::state::{CANCELLED, COMPLETED};

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};

/// A handle that allows awaiting the result of a spawned task.
///
/// A `JoinHandle` is returned by the `spawn` function and represents a
/// handle to a task running in the background. It implements [`Future`],
/// allowing you to `.await` the task's return value.
///
/// # Panics
///
/// Polling a `JoinHandle` after the task has been aborted will result in a panic.
/// Additionally, attempting to poll a `JoinHandle` after it has already returned
/// [`Poll::Ready`] will panic, as the task result is consumed upon completion.
pub struct JoinHandle<T> {
    /// A shared reference to the underlying task and its state.
    pub(crate) task: Arc<Task<T>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    /// Polls the task for completion, returning the result if ready.
    ///
    /// This implementation employs a "double-check" synchronization pattern to
    /// ensure that no wake-ups are missed between the state check and waker
    /// registration.
    ///
    /// ### State Machine Logic:
    /// 1. **Initial Check**: If the task is `COMPLETED`, the result is taken and returned.
    /// 2. **Waker Registration**: If not ready, the current [`Waker`](std::task::Waker)
    ///    is added to the task's waiter list.
    /// 3. **Secondary Check**: The state is checked again. This handles the race
    ///    condition where the task completes exactly between the first check
    ///    and the registration.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // --- Phase 1: Initial state check ---
        let state = self.task.state.load(Ordering::Acquire);

        if state == COMPLETED {
            let value = unsafe {
                (*self.task.result.get())
                    .take()
                    .expect("task result was already consumed; JoinHandle cannot be polled twice")
            };
            return Poll::Ready(value);
        }

        if state == CANCELLED {
            panic!("JoinHandle polled after task was aborted");
        }

        // --- Phase 2: Register interest in task completion ---
        // We lock the waiters list to safely push the waker.
        self.task.waiters.lock().unwrap().push(cx.waker().clone());

        // --- Phase 3: Secondary check (Double-Check Pattern) ---
        // This prevents the "lost wake-up" problem. If the task finished
        // after our first check but before we pushed the waker, the task's
        // `run` method wouldn't have seen our waker. We check again to be sure.
        let state_after = self.task.state.load(Ordering::Acquire);
        if state_after == COMPLETED {
            let value = unsafe {
                (*self.task.result.get())
                    .take()
                    .expect("task result was already consumed")
            };
            return Poll::Ready(value);
        }

        if state_after == CANCELLED {
            panic!("JoinHandle polled after task was aborted");
        }

        Poll::Pending
    }
}

impl<T: Send + 'static> SetHandle for JoinHandle<T> {
    /// Polls the handle specifically for the `JoinSet` internal management logic.
    ///
    /// This method allows the `JoinSet` to drive the task to completion without
    /// needing to know the specific return type `T`. It delegates the poll to
    /// the underlying future and converts any result to `()`.
    fn poll_completed(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match Future::poll(self, cx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Triggers the abort logic on the underlying task.
    ///
    /// This will transition the task state to `CANCELLED` and notify any
    /// registered wakers.
    fn abort(&self) {
        self.task.abort();
    }
}
