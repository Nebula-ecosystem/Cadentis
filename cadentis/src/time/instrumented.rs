use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Wraps a future and measures the time it takes to complete.
///
/// The returned future resolves to a tuple containing:
/// - the output of the wrapped future,
/// - the elapsed time since the first poll.
///
/// Timing starts on the **first poll**, not at construction time.
///
/// # Examples
///
/// ```rust,ignore
/// let (value, elapsed) = instrumented(async { 42 }).await;
/// println!("Completed in {:?}", elapsed);
/// ```
pub fn instrumented<F>(future: F) -> Instrumented<F> {
    Instrumented::new(future)
}

/// A future that measures the execution time of another future.
///
/// This type is lazy: the timer starts when the future is first polled.
/// Dropping the future before completion discards the measurement.
pub struct Instrumented<F> {
    /// The wrapped future.
    future: F,

    /// Instant marking the first poll.
    start: Option<Instant>,
}

impl<F> Instrumented<F> {
    /// Creates a new `Instrumented` future.
    fn new(future: F) -> Self {
        Self {
            future,
            start: None,
        }
    }
}

impl<F: Future> Future for Instrumented<F> {
    /// Returns the output of the future and the elapsed duration.
    type Output = (F::Output, Duration);

    /// Polls the instrumented future.
    ///
    /// On the first poll, the start time is recorded. Once the wrapped
    /// future completes, the elapsed duration is returned alongside
    /// the output.
    ///
    /// # Safety
    ///
    /// This implementation uses `unsafe` pin projections but is sound
    /// because the wrapped future is never moved after being pinned.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        let start = *this.start.get_or_insert_with(Instant::now);

        let res = unsafe { Pin::new_unchecked(&mut this.future) }.poll(cx);

        match res {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => {
                let elapsed = start.elapsed();
                Poll::Ready((output, elapsed))
            }
        }
    }
}
