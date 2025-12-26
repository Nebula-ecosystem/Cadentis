//! Cooperative task yielding.
//!
//! Provides a mechanism for tasks to yield control to other tasks, allowing
//! fair scheduling in a cooperative multitasking environment.
//!
//! # Example
//!
//! ```ignore
//! use reactor::yield_now;
//!
//! async fn cooperative_task() {
//!     for i in 0..100 {
//!         println!("Iteration {}", i);
//!         
//!         // Yield to let other tasks run
//!         yield_now().await;
//!     }
//! }
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Cooperative scheduler hint: yields once to let other tasks run.
///
/// This returns a future that yields [`Poll::Pending`] the first time it's polled
/// and immediately schedules the current task to be polled again by calling
/// `cx.waker().wake_by_ref()`. On the second poll, it returns [`Poll::Ready`].
///
/// This is similar to `tokio::task::yield_now()`.
///
/// # Example
///
/// ```ignore
/// use reactor::yield_now;
///
/// async fn long_running() {
///     for i in 0..1000 {
///         // Do some work
///         process(i);
///         
///         // Yield periodically to be fair to other tasks
///         if i % 10 == 0 {
///             yield_now().await;
///         }
///     }
/// }
/// ```
///
/// [`Poll::Pending`]: std::task::Poll::Pending
/// [`Poll::Ready`]: std::task::Poll::Ready
pub async fn yield_now() {
    /// Future that yields once before completing.
    struct YieldOnce(bool);

    impl Future for YieldOnce {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.0 {
                self.0 = true;
                cx.waker().wake_by_ref();

                return Poll::Pending;
            }

            Poll::Ready(())
        }
    }

    YieldOnce(false).await
}
