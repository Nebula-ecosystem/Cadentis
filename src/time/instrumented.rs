use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub fn instrumented<F>(future: F) -> Instrumented<F> {
    Instrumented::new(future)
}

pub struct Instrumented<F> {
    future: F,
    elapsed_ns: AtomicU64,
}

impl<F> Instrumented<F> {
    pub fn new(future: F) -> Self {
        Self {
            future,
            elapsed_ns: AtomicU64::new(0),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_nanos(self.elapsed_ns.load(Ordering::Relaxed))
    }
}

impl<F: Future> Future for Instrumented<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let start = Instant::now();

        let res = unsafe { Pin::new_unchecked(&mut this.future) }.poll(cx);

        let elapsed = start.elapsed();
        this.elapsed_ns
            .fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);

        res
    }
}
