use crate::time::sleep::{Sleep, sleep};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// Creates a future that completes with an error if the given duration
/// elapses before the provided future completes.
///
/// If the wrapped future completes first, its output is returned inside
/// `Ok`. If the timeout expires first, `Err(())` is returned.
///
/// # Arguments
///
/// * `duration` - Maximum amount of time to wait for the future.
/// * `future` - The future to execute with a time limit.
///
/// # Examples
///
/// ```rust,ignore
/// use std::time::Duration;
///
/// async fn work() -> u32 {
///     42
/// }
///
/// let result = timeout(Duration::from_secs(1), work()).await;
/// assert_eq!(result, Ok(42));
/// ```
pub fn timeout<F>(duration: Duration, future: F) -> Timeout<F>
where
    F: Future,
{
    Timeout::new(duration, future)
}

/// A future that enforces a time limit on another future.
///
/// `Timeout` polls both the wrapped future and an internal sleep future.
/// Whichever completes first determines the result.
///
/// This type is lazy: neither the wrapped future nor the timer makes
/// progress until `poll` is called.
pub struct Timeout<F> {
    /// The wrapped future whose execution is being timed.
    future: F,

    /// Sleep future used to track the timeout.
    sleep: Sleep,
}

impl<F> Timeout<F> {
    /// Creates a new `Timeout` future.
    ///
    /// The timer starts counting once the future is first polled.
    pub(crate) fn new(duration: Duration, future: F) -> Self {
        Timeout {
            future,
            sleep: sleep(duration),
        }
    }
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    /// Returns `Ok(output)` if the future completes in time,
    /// or `Err(())` if the timeout expires first.
    type Output = Result<F::Output, ()>;

    /// Polls the timeout future.
    ///
    /// This method first polls the wrapped future. If it is still
    /// pending, the internal timer is then polled.
    ///
    /// # Safety
    ///
    /// This implementation uses `unsafe` pin projections but is sound
    /// because:
    /// - `future` is never moved after being pinned
    /// - `sleep` is never moved after being pinned
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        if let Poll::Ready(val) = future.poll(cx) {
            return Poll::Ready(Ok(val));
        }

        let sleep = unsafe { Pin::new_unchecked(&mut this.sleep) };
        if let Poll::Ready(()) = sleep.poll(cx) {
            return Poll::Ready(Err(()));
        }

        Poll::Pending
    }
}
