use crate::time::sleep::{Sleep, sleep};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pub fn timeout<F>(duration: Duration, future: F) -> Timeout<F>
where
    F: Future,
{
    Timeout::new(duration, future)
}

pub struct Timeout<F> {
    future: F,
    sleep: Sleep,
}

impl<F> Timeout<F> {
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
    type Output = Result<F::Output, ()>;

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
