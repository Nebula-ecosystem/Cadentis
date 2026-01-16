use crate::reactor::command::Command;
use crate::runtime::context::CURRENT_REACTOR;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Creates a future that completes after the given duration.
///
/// The returned sleep future registers a timer with the current
/// runtime reactor and completes once the duration has elapsed.
///
/// # Panics
///
/// Panics if polled outside of a running runtime.
///
/// # Examples
///
/// ```rust,ignore
/// use std::time::Duration;
///
/// sleep(Duration::from_millis(10)).await;
/// ```
pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

/// A future that completes once a specific deadline is reached.
///
/// `Sleep` integrates with the runtime reactor by registering a timer
/// command on first poll. The timer is automatically cancelled if the
/// future is dropped before completion.
///
/// This future is **cancel-safe**: dropping it will prevent the timer
/// from waking the task.
pub struct Sleep {
    /// Absolute point in time when the sleep completes.
    deadline: Instant,

    /// Whether the timer has already been registered with the reactor.
    registered: bool,

    /// Cancellation flag shared with the reactor.
    cancelled: Arc<AtomicBool>,
}

impl Sleep {
    /// Creates a new `Sleep` future that completes after `duration`.
    ///
    /// The timer is not registered until the future is first polled.
    pub(crate) fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            registered: false,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Future for Sleep {
    /// The sleep future produces no value.
    type Output = ();

    /// Polls the sleep future.
    ///
    /// On the first poll, the timer is registered with the reactor.
    /// The task is woken once the deadline is reached or if the
    /// sleep is cancelled.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.cancelled.load(Ordering::Acquire) || Instant::now() >= this.deadline {
            return Poll::Ready(());
        }

        if !this.registered {
            this.registered = true;

            CURRENT_REACTOR.with(|cell| {
                let binding = cell.borrow();
                let reactor = binding.as_ref().expect("Sleep polled outside of runtime");

                let _ = reactor.send(Command::SetTimer {
                    deadline: this.deadline,
                    waker: cx.waker().clone(),
                    cancelled: this.cancelled.clone(),
                });
            });
        }

        Poll::Pending
    }
}

impl Drop for Sleep {
    /// Cancels the timer if the sleep future is dropped before completion.
    ///
    /// This ensures that no spurious wake-ups occur after the future
    /// has been abandoned.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
