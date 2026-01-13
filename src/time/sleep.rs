use crate::reactor::command::Command;
use crate::runtime::context::CURRENT_REACTOR;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

pub struct Sleep {
    deadline: Instant,
    registered: bool,
    cancelled: Arc<AtomicBool>,
}

impl Sleep {
    pub(crate) fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            registered: false,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.cancelled.load(Ordering::Acquire) || Instant::now() >= this.deadline {
            return Poll::Ready(());
        }

        if !this.registered {
            this.registered = true;

            CURRENT_REACTOR.with(|cell| {
                let binding = cell.borrow();
                let reactor = binding.as_ref().expect("Sleep polled outside of runtime");

                let _ = reactor.send(Command::SetTimer {
                    deadline: this.deadline,
                    waker: cx.waker().clone(),
                    cancelled: this.cancelled.clone(),
                });

                let _ = reactor.send(Command::Wake);
            });
        }

        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
