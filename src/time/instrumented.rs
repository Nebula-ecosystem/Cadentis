use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub fn instrumented<F>(future: F) -> Instrumented<F> {
    Instrumented::new(future)
}

pub struct Instrumented<F> {
    future: F,
    start: Option<Instant>,
}

impl<F> Instrumented<F> {
    fn new(future: F) -> Self {
        Self {
            future,
            start: None,
        }
    }
}

impl<F: Future> Future for Instrumented<F> {
    type Output = (F::Output, Duration);

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
