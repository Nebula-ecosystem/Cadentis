use super::state::{COMPLETED, IDLE, QUEUED, RUNNING};
use crate::runtime::context::{CURRENT_INJECTOR, CURRENT_LOCALS, CURRENT_WORKER_ID};
use crate::runtime::task::waker::make_waker;
use crate::runtime::work_stealing::injector::Injector;
use crate::task::JoinHandle;

use std::cell::UnsafeCell;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub(crate) trait Runnable: Send + Sync {
    fn run(self: Arc<Self>);
}

pub(crate) struct Task<T> {
    future: UnsafeCell<Pin<Box<dyn Future<Output = T> + Send>>>,

    pub(crate) result: UnsafeCell<Option<T>>,
    pub(crate) state: AtomicUsize,

    injector: Arc<Injector>,
    pub(crate) waiters: Mutex<Vec<Waker>>,
}

unsafe impl<T> Send for Task<T> {}
unsafe impl<T> Sync for Task<T> {}

impl<T: Send + 'static> Task<T> {
    pub(crate) fn new<F>(future: F, injector: Arc<Injector>) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            future: UnsafeCell::new(Box::pin(future)),
            result: UnsafeCell::new(None),
            state: AtomicUsize::new(QUEUED),
            injector,
            waiters: Mutex::new(Vec::new()),
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

            Poll::Ready(val) => {
                unsafe {
                    *self.result.get() = Some(val);
                }

                self.state.store(COMPLETED, Ordering::Release);

                let waiters = self.waiters.lock().unwrap();
                for w in waiters.iter() {
                    w.wake_by_ref();
                }
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

impl<T: Send + 'static> Runnable for Task<T> {
    fn run(self: Arc<Self>) {
        Task::run(self)
    }
}

pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
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
        injector.push(task.clone());
    }

    JoinHandle { task }
}
