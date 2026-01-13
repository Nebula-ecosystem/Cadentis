use super::state::{COMPLETED, IDLE, QUEUED, RUNNING};
use crate::runtime::context::{CURRENT_INJECTOR, CURRENT_LOCALS, CURRENT_WORKER_ID};
use crate::runtime::task::waker::make_waker;
use crate::runtime::work_stealing::injector::Injector;

use std::cell::UnsafeCell;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};

pub(crate) struct Task {
    future: UnsafeCell<Pin<Box<dyn Future<Output = ()> + Send>>>,
    pub(crate) state: AtomicUsize,
    injector: Arc<Injector>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub(crate) fn new<F>(future: F, injector: Arc<Injector>) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            future: UnsafeCell::new(Box::pin(future)),
            state: AtomicUsize::new(QUEUED),
            injector,
        }
    }

    pub(crate) fn run(self: Arc<Self>) {
        if self
            .state
            .compare_exchange(QUEUED, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        let waker = make_waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let poll = unsafe { (&mut *self.future.get()).as_mut().poll(&mut cx) };

        match poll {
            Poll::Pending => {
                self.state.store(IDLE, Ordering::Release);
            }

            Poll::Ready(()) => {
                self.state.store(COMPLETED, Ordering::Release);
            }
        }
    }

    pub fn wake(self: Arc<Self>) {
        if self
            .state
            .compare_exchange(IDLE, QUEUED, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.injector.push(self.clone());
        }
    }
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let injector = CURRENT_INJECTOR.with(|cell| {
        cell.borrow()
            .as_ref()
            .expect("spawn called outside of runtime")
            .clone()
    });

    let task = Arc::new(Task::new(future, injector.clone()));

    let pushed_locally = CURRENT_WORKER_ID.with(|id_cell| {
        let id = *id_cell.borrow();
        if let Some(id) = id {
            CURRENT_LOCALS.with(|locals_cell| {
                if let Some(locals) = locals_cell.borrow().as_ref() {
                    locals[id].push(task.clone());
                    return true;
                }
                false
            })
        } else {
            false
        }
    });

    if !pushed_locally {
        injector.push(task);
    }
}
