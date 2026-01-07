use crate::runtime::{CURRENT_QUEUE, TaskQueue, make_waker};

use std::cell::UnsafeCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

pub struct Task<T: Send + Sync + 'static> {
    pub(crate) future: UnsafeCell<Pin<Box<dyn Future<Output = T> + Send>>>,

    pub(crate) result: UnsafeCell<Option<T>>,
    pub(crate) completed: AtomicBool,

    pub(crate) queue: Arc<TaskQueue>,
    pub(crate) inqueue: AtomicBool,

    pub(crate) waiter: UnsafeCell<Vec<Waker>>,
}

unsafe impl<T> Sync for Task<T> where T: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Task<T> {
    pub(crate) fn new<F>(fut: F, queue: Arc<TaskQueue>) -> Arc<Self>
    where
        F: Future<Output = T> + Send + 'static,
    {
        Arc::new(Task {
            future: UnsafeCell::new(Box::pin(fut)),
            result: UnsafeCell::new(None),
            queue,
            inqueue: AtomicBool::new(true),
            completed: AtomicBool::new(false),
            waiter: UnsafeCell::new(Vec::new()),
        })
    }

    pub fn poll(self: &Arc<Self>) {
        if self.completed.swap(false, Ordering::AcqRel) {
            return;
        }

        let waker = make_waker(self.clone());
        let mut context = Context::from_waker(&waker);

        let future = unsafe { &mut *self.future.get() };

        match future.as_mut().poll(&mut context) {
            Poll::Pending => {
                if !self.inqueue.swap(true, Ordering::AcqRel) {
                    self.queue.push(self.clone());
                }
            }
            Poll::Ready(val) => {
                unsafe {
                    *self.result.get() = Some(val);
                }

                self.completed.store(true, Ordering::Release);
                self.inqueue.store(false, Ordering::Release);

                unsafe { (*self.waiter.get()).drain(..) }.for_each(|w| w.wake());
            }
        }
    }

    pub fn spawn<F>(future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
    {
        CURRENT_QUEUE.with(|current| {
            let queue = current
                .borrow()
                .as_ref()
                .expect("Task::spawn() called outside of a runtime context")
                .clone();

            let task = Task::new(future, queue.clone());
            let runnable: Arc<dyn Runnable> = task.clone();

            queue.push(runnable);

            JoinHandle { task }
        })
    }
}

pub(crate) trait Runnable: Send + Sync {
    fn poll(self: Arc<Self>);
}

impl<T: Send + Sync + 'static> Runnable for Task<T> {
    fn poll(self: Arc<Self>) {
        Task::poll(&self);
    }
}

pub struct JoinHandle<T: Send + Sync + 'static> {
    task: Arc<Task<T>>,
}

impl<T: Send + Sync> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task.completed.load(Ordering::SeqCst) {
            let result = unsafe { (*self.task.result.get()).take() }.unwrap();

            return Poll::Ready(result);
        }

        unsafe { &mut *self.task.waiter.get() }.push(cx.waker().clone());

        Poll::Pending
    }
}
