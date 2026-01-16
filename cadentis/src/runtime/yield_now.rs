use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future that yields execution back to the executor exactly once.
struct YieldOnce(bool);

impl Future for YieldOnce {
    type Output = ();

    /// Polls the yield future.
    ///
    /// On the first poll, the task yields by scheduling itself to be
    /// polled again and returning `Poll::Pending`.
    /// On the second poll, the future completes.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        Poll::Ready(())
    }
}

/// Yields execution back to the executor.
///
/// This allows other tasks to make progress before the current task
/// continues. The function yields exactly once.
///
/// # Examples
///
/// ```rust,ignore
/// async fn task() {
///     // Allow other tasks to run
///     yield_now().await;
/// }
/// ```
pub async fn yield_now() {
    YieldOnce(false).await
}
