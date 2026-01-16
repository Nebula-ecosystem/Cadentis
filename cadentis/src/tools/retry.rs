use crate::time::sleep;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// Creates a future that retries an asynchronous operation on failure.
///
/// The operation is produced by a factory function, which is called again
/// each time a retry is needed. The retry stops early if the future resolves
/// successfully.
///
/// # Arguments
///
/// * `times` - Number of retry attempts after the first failure.
/// * `factory` - A closure producing a new future on each attempt.
///
/// # Examples
///
/// ```rust,ignore
/// async fn fallible() -> Result<u32, ()> {
///     Err(())
/// }
///
/// let retry = retry(3, || fallible());
/// ```
pub fn retry<F, G>(times: usize, factory: G) -> Retry<G, F>
where
    G: FnMut() -> F + Send + 'static,
    F: Future + Send + 'static,
{
    Retry::new(times, factory)
}

/// A future that retries an asynchronous operation until it succeeds
/// or the retry limit is reached.
///
/// Each retry creates a fresh future using the provided factory.
/// An optional delay can be configured between retries using
/// [`set_interval`](Self::set_interval).
///
/// This type is lazy: no future is created until it is first polled.
pub struct Retry<G, F> {
    /// Factory used to create a new future for each attempt.
    factory: G,

    /// Currently running future.
    future: Option<Pin<Box<F>>>,

    /// Optional delay future between retries.
    delay: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    /// Number of remaining retries.
    remaining: usize,

    /// Delay interval between retries.
    interval: Duration,
}

impl<G, F> Retry<G, F> {
    /// Creates a new `Retry` future.
    ///
    /// The retry interval is initially zero (no delay).
    fn new(times: usize, factory: G) -> Self {
        Self {
            factory,
            future: None,
            delay: None,
            remaining: times,
            interval: Duration::from_micros(0),
        }
    }

    /// Sets a delay between retry attempts.
    ///
    /// If the interval is zero, retries occur immediately.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// let retry = retry(5, || async { Err::<(), ()>(()) })
    ///     .set_interval(Duration::from_millis(100));
    /// ```
    pub fn set_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

impl<G, F, T, E> Future for Retry<G, F>
where
    G: FnMut() -> F + Send + Unpin + 'static,
    F: Future<Output = Result<T, E>> + Send + 'static,
{
    type Output = Result<T, E>;

    /// Polls the retry future.
    ///
    /// The future:
    /// - resolves immediately on the first successful attempt,
    /// - retries on error until the retry count is exhausted,
    /// - optionally waits for the configured interval between attempts.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let Some(delay) = this.delay.as_mut() {
            match delay.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(()) => {
                    this.delay = None;
                }
            }
        }

        if this.future.is_none() {
            let fut = (this.factory)();
            this.future = Some(Box::pin(fut));
        }

        let fut = this.future.as_mut().unwrap();

        match fut.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,

            Poll::Ready(Ok(v)) => {
                this.future = None;
                Poll::Ready(Ok(v))
            }

            Poll::Ready(Err(e)) => {
                this.future = None;

                if this.remaining > 0 {
                    this.remaining -= 1;

                    if this.interval != Duration::from_micros(0) {
                        this.delay = Some(Box::pin(sleep(this.interval)));
                    }

                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(Err(e))
                }
            }
        }
    }
}
