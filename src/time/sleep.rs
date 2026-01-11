use crate::reactor::command::Command;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration, _)
}

pub struct Sleep {
    deadline: Instant,
    registered: bool,
    cancelled: Arc<AtomicBool>,
    sender: Sender<Command>,
}

impl Sleep {
    pub(crate) fn new(duration: Duration, sender: Sender<Command>) -> Self {
        let deadline = Instant::now() + duration;

        Self {
            deadline,
            registered: false,
            cancelled: Arc::new(AtomicBool::new(false)),
            sender,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.cancelled.load(Ordering::Acquire) || Instant::now() >= self.deadline {
            return Poll::Ready(());
        }

        if !self.registered {
            self.registered = true;
            self.sender.send(Command::SetTimer {
                deadline: self.deadline,
                waker: cx.waker().clone(),
                cancelled: self.cancelled.clone(),
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
