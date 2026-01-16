use super::state::{COMPLETED, IDLE, NOTIFIED, QUEUED, RUNNING};
use crate::runtime::context::{CURRENT_INJECTOR, CURRENT_LOCALS, CURRENT_WORKER_ID};
use crate::runtime::task::waker::make_waker;
use crate::runtime::work_stealing::injector::Injector;
use crate::task::JoinHandle;

use std::cell::UnsafeCell;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// A runnable unit of work.
///
/// Types implementing `Runnable` can be scheduled and executed
/// by the executor. This abstraction allows the scheduler to
/// operate on heterogeneous task types.
///
/// In practice, `Runnable` is implemented by [`Task`].
pub(crate) trait Runnable: Send + Sync {
    /// Executes the runnable task.
    fn run(self: Arc<Self>);
}

/// A spawned asynchronous task.
///
/// A `Task` wraps a future and manages:
/// - its execution state,
/// - wake-up logic,
/// - result storage,
/// - coordination with the scheduler.
///
/// Tasks are reference-counted and can be safely shared across
/// worker threads.
pub(crate) struct Task<T> {
    /// The future being executed.
    ///
    /// Stored in an `UnsafeCell` to allow mutable access across
    /// poll invocations while respecting pinning guarantees.
    future: UnsafeCell<Pin<Box<dyn Future<Output = T> + Send>>>,

    /// Storage for the task result once completed.
    pub(crate) result: UnsafeCell<Option<T>>,

    /// Current execution state of the task.
    pub(crate) state: AtomicUsize,

    /// Handle to the global injector queue.
    injector: Arc<Injector>,

    /// Wakers waiting for the task to complete (used by `JoinHandle`).
    pub(crate) waiters: Mutex<Vec<Waker>>,
}

unsafe impl<T> Send for Task<T> {}
unsafe impl<T> Sync for Task<T> {}

impl<T: Send + 'static> Task<T> {
    /// Creates a new task from a future.
    ///
    /// Newly created tasks start in the `QUEUED` state.
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

    /// Executes the task.
    ///
    /// This method:
    /// - transitions the task into the `RUNNING` state,
    /// - polls the future,
    /// - handles state transitions based on the poll result,
    /// - wakes join handles upon completion.
    pub(crate) fn run(self: Arc<Self>) {
        let current = self.state.load(Ordering::Acquire);

        if current != QUEUED && current != NOTIFIED {
            return;
        }

        if self
            .state
            .compare_exchange(current, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        let waker = make_waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let poll = unsafe { (&mut *self.future.get()).as_mut().poll(&mut cx) };

        match poll {
            // The task is not yet complete.
            Poll::Pending => {
                if self
                    .state
                    .compare_exchange(RUNNING, IDLE, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    // The task was notified while running.
                    self.state.store(QUEUED, Ordering::Release);
                    self.injector.push(self.clone());
                }
            }

            // The task has completed.
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

    /// Wakes the task.
    ///
    /// This method is called via the task's waker and is responsible
    /// for rescheduling the task according to its current state.
    pub fn wake(self: Arc<Self>) {
        loop {
            let state = self.state.load(Ordering::Acquire);

            match state {
                // Idle tasks are re-queued for execution.
                IDLE => {
                    if self
                        .state
                        .compare_exchange(IDLE, QUEUED, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        self.injector.push(self.clone());
                        return;
                    }
                }

                // Running tasks record that they were notified.
                RUNNING => {
                    if self
                        .state
                        .compare_exchange(RUNNING, NOTIFIED, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return;
                    }
                }

                // Already scheduled or completed tasks require no action.
                QUEUED | NOTIFIED | COMPLETED => {
                    return;
                }

                _ => return,
            }
        }
    }
}

impl<T: Send + 'static> Runnable for Task<T> {
    /// Executes the task as a runnable unit.
    fn run(self: Arc<Self>) {
        Task::run(self)
    }
}

/// Spawns a new asynchronous task onto the current runtime.
///
/// The task is scheduled either:
/// - onto the current worker's local queue (if called from a worker),
/// - or into the global injector queue otherwise.
///
/// # Panics
///
/// Panics if called outside of a running runtime.
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
