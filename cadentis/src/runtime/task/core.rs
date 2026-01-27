use super::JoinHandle;
use super::state::{CANCELLED, COMPLETED, IDLE, NOTIFIED, QUEUED, RUNNING};
use crate::runtime::context::{CURRENT_INJECTOR, CURRENT_LOCALS, CURRENT_WORKER_ID};
use crate::runtime::task::waker::make_waker;
use crate::runtime::work_stealing::injector::Injector;

use std::cell::UnsafeCell;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// A runnable unit of work that can be executed by the scheduler.
///
/// The `Runnable` trait abstracts the specific return type of a task,
/// allowing the executor to manage a heterogeneous collection of tasks
/// through `Arc<dyn Runnable>`.
pub(crate) trait Runnable: Send + Sync {
    /// Executes the task. This is typically called by a worker thread.
    fn run(self: Arc<Self>);
}

/// A spawned asynchronous task managed by the runtime.
///
/// A `Task` acts as the container for a `Future`. It coordinates the lifecycle
/// of that future, including its execution state, waker registration,
/// and result storage.
pub(crate) struct Task<T> {
    /// The underlying future.
    ///
    /// Wrapped in `UnsafeCell` for interior mutability during `poll`, and
    /// `Pin<Box<...>>` to ensure the future remains pinned in memory.
    future: UnsafeCell<Pin<Box<dyn Future<Output = T> + Send>>>,

    /// Storage for the result produced by the future upon completion.
    pub(crate) result: UnsafeCell<Option<T>>,

    /// The current lifecycle state of the task (IDLE, RUNNING, etc.).
    pub(crate) state: AtomicUsize,

    /// Reference to the global injector queue for rescheduling.
    injector: Arc<Injector>,

    /// A list of wakers belonging to `JoinHandle`s awaiting this task.
    pub(crate) waiters: Mutex<Vec<Waker>>,
}

unsafe impl<T> Send for Task<T> {}
unsafe impl<T> Sync for Task<T> {}

impl<T: Send + 'static> Task<T> {
    /// Creates a new task instance from a future.
    ///
    /// The task is initialized in the `QUEUED` state, indicating it is ready
    /// to be processed by the scheduler.
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

    /// Performs the execution of the task.
    ///
    /// This method transitions the task to `RUNNING`, polls the inner future,
    /// and handles the resulting `Poll` state:
    /// - `Poll::Pending`: Transitions back to `IDLE` or re-queues if notified.
    /// - `Poll::Ready`: Stores the result and notifies all `JoinHandle` waiters.
    pub(crate) fn run(self: Arc<Self>) {
        let current = self.state.load(Ordering::Acquire);

        // Early exit if the task is already cancelled or is not in a runnable state.
        if current == CANCELLED || (current != QUEUED && current != NOTIFIED) {
            return;
        }

        // Transition to RUNNING. This ensures exclusive access to the UnsafeCell.
        if self
            .state
            .compare_exchange(current, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        let waker = make_waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // Safety: The RUNNING state guarantees that no other thread is polling this future.
        let poll = unsafe { (&mut *self.future.get()).as_mut().poll(&mut cx) };

        match poll {
            Poll::Pending => {
                // Return to IDLE state unless a wake-up occurred during execution (NOTIFIED).
                if self
                    .state
                    .compare_exchange(RUNNING, IDLE, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    // Task was notified while running; move back to QUEUED and reschedule.
                    self.state.store(QUEUED, Ordering::Release);
                    self.injector.push(self.clone());
                }
            }
            Poll::Ready(val) => {
                // Store the result and finalize the task state.
                unsafe {
                    *self.result.get() = Some(val);
                }
                self.state.store(COMPLETED, Ordering::Release);

                // Wake all handles awaiting the result of this task.
                let waiters = self.waiters.lock().unwrap();
                for w in waiters.iter() {
                    w.wake_by_ref();
                }
            }
        }
    }

    /// Signals the task to be rescheduled.
    ///
    /// If the task is `IDLE`, it moves to `QUEUED` and is pushed to the scheduler.
    /// If the task is `RUNNING`, it moves to `NOTIFIED` to ensure it is re-polled
    /// immediately after its current execution slice.
    pub fn wake(self: Arc<Self>) {
        loop {
            let state = self.state.load(Ordering::Acquire);

            match state {
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
                RUNNING => {
                    if self
                        .state
                        .compare_exchange(RUNNING, NOTIFIED, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return;
                    }
                }
                // If the task is already queued, notified, or finished, no action is needed.
                QUEUED | NOTIFIED | COMPLETED | CANCELLED => return,
                _ => return,
            }
        }
    }

    /// Aborts the task execution.
    ///
    /// Transitions the task to the `CANCELLED` state. If the task transitions
    /// successfully, all waiters are notified so they can stop awaiting the result.
    pub fn abort(&self) {
        loop {
            let state = self.state.load(Ordering::Acquire);

            // Cannot abort a task that is already finished or already cancelled.
            if state == COMPLETED || state == CANCELLED {
                return;
            }

            if self
                .state
                .compare_exchange(state, CANCELLED, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // Notify waiters so they can observe the cancellation state.
                let waiters = self.waiters.lock().unwrap();
                for w in waiters.iter() {
                    w.wake_by_ref();
                }
                break;
            }
        }
    }
}

impl<T: Send + 'static> Runnable for Task<T> {
    fn run(self: Arc<Self>) {
        Task::run(self)
    }
}

/// Spawns a future as a task onto the current runtime.
///
/// The task is first attempted to be pushed to the local worker's queue
/// for better cache locality. If called from outside the runtime, it is
/// pushed to the global injector queue.
///
/// # Panics
/// Panics if called outside the context of a running runtime.
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
{
    let injector = CURRENT_INJECTOR.with(|cell| {
        cell.borrow()
            .as_ref()
            .expect("spawn must be called within the context of a runtime")
            .clone()
    });

    let task = Arc::new(Task::new(future, injector.clone()));

    // Try local queue injection for performance.
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

    // Fallback to global injector.
    if !pushed_locally {
        injector.push(task.clone());
    }

    JoinHandle { task }
}
