use crate::time::sleep;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pub fn retry<F, G>(times: usize, factory: G) -> Retry<G, F>
where
    G: FnMut() -> F + Send + 'static,
    F: Future + Send + 'static,
{
    Retry::new(times, factory)
}

pub struct Retry<G, F> {
    factory: G,
    future: Option<Pin<Box<F>>>,

    delay: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    remaining: usize,
    interval: Duration,
}

impl<G, F> Retry<G, F> {
    fn new(times: usize, factory: G) -> Self {
        Self {
            factory,
            future: None,
            delay: None,
            remaining: times,
            interval: Duration::from_micros(0),
        }
    }

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
