use crate::time::sleep;

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// Creates a future that retries the provided asynchronous operation up to a given number of times.
///
/// # Arguments
///
/// * `times` - The maximum number of retry attempts.
/// * `factory` - A closure that returns a new future for each attempt.
///
/// # Returns
///
/// A `Retry` future that resolves to the result of the operation or an error if all attempts fail.
pub fn retry<F, G>(times: usize, factory: G) -> Retry<G, F>
where
    G: FnMut() -> F,
    F: Future,
{
    Retry::new(times, factory)
}

/// A future that retries an asynchronous operation multiple times.
///
/// This wrapper recreates the future on each retry using the provided factory closure.
/// If the operation fails, it will be retried up to the specified number of times.
pub struct Retry<G, F> {
    factory: G,
    future: Option<Pin<Box<F>>>,
    delay: Option<Pin<Box<dyn Future<Output = ()>>>>,
    remaining: usize,
    interval: Duration,
}

impl<G, F> Retry<G, F> {
    /// Creates a new `Retry` future.
    ///
    /// # Arguments
    ///
    /// * `times` - The maximum number of retry attempts.
    /// * `factory` - A closure that returns a new future for each attempt.
    fn new(times: usize, factory: G) -> Self {
        Self {
            factory,
            future: None,
            delay: None,
            remaining: times,
            interval: Duration::from_micros(0),
        }
    }

    /// Sets the interval to wait between retry attempts.
    ///
    /// # Arguments
    ///
    /// * `interval` - The duration to wait between each retry attempt.
    ///
    /// # Returns
    ///
    /// The `Retry` future with the interval configured.
    pub fn set_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

impl<G, F, T, E> Future for Retry<G, F>
where
    G: FnMut() -> F,
    F: Future<Output = Result<T, E>> + 'static,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if let Some(delay) = this.delay.as_mut() {
            match delay.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(()) => {
                    this.delay = None;
                }
            }
        }

        if this.future.is_none() {
            this.future = Some(Box::pin((this.factory)()));
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
