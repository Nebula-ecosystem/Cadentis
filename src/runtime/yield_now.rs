use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub async fn yield_now() {
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
