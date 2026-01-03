use crate::reactor::core::ReactorHandle;
use crate::runtime::context::current_reactor_io;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pub struct Sleep {
    duration: Duration,
    reactor: ReactorHandle,
    registered: bool,
}

impl Sleep {
    pub(crate) fn new(duration: Duration) -> Self {
        Self::new_with_reactor(duration, current_reactor_io())
    }

    pub(crate) fn new_with_reactor(duration: Duration, reactor: ReactorHandle) -> Self {
        Self {
            duration,
            reactor,
            registered: false,
        }
    }
}

pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.duration.is_zero() {
            return Poll::Ready(());
        }

        if !self.registered {
            let waker = cx.waker().clone();
            self.reactor
                .lock()
                .unwrap()
                .register_timer(self.duration, waker);
            self.registered = true;

            return Poll::Pending;
        }

        Poll::Ready(())
    }
}
