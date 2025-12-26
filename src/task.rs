//! Task wrapper that combines futures with waker integration.
//!
//! A task encapsulates a future and provides mechanisms for polling and awakening
//! when the future is ready to make progress. Supports both direct task execution via
//! the runtime and global task spawning without requiring an explicit runtime reference.
//!
//! # Task Spawning
//!
//! Tasks are spawned using [`Task::spawn`] from within an async context:
//!
//! ```ignore
//! use reactor::Task;
//!
//! async fn spawn_example() {
//!     Task::spawn(async {
//!         println!("Running in background");
//!     });
//!     println!("Task spawned, main continues");
//! }
//! ```
//!
//! # Join Handles
//!
//! [`Task::spawn`] returns a [`JoinHandle`] that can be awaited to wait for completion:
//!
//! ```ignore
//! use reactor::Task;
//!
//! async fn wait_example() {
//!     let handle = Task::spawn(async { 42 });
//!     let result = handle.await;
//!     println!("Task completed");
//! }
//! ```
//!
//! # JoinSet
//!
//! Use [`JoinSet`] to collect multiple handles and await them all:
//!
//! ```ignore
//! use reactor::{Task, JoinSet};
//!
//! async fn join_set_example() {
//!     let mut set = JoinSet::new();
//!     for i in 0..10 {
//!         set.push(Task::spawn(async move {
//!             println!("Task {}", i);
//!         }));
//!     }
//!     set.await_all().await;
//!     println!("All tasks completed");
//! }
//! ```
//!
//! # How Tasks Work
//!
//! 1. A future is wrapped in a [`Task`]
//! 2. The task is enqueued in the runtime's task queue
//! 3. The executor polls the task with a custom waker
//! 4. When the future yields `Poll::Pending`, it's stored for later
//! 5. When an I/O event or timer fires, the waker re-queues the task
//! 6. The task is polled again and can make progress

use crate::runtime::{CURRENT_QUEUE, TaskQueue, make_waker};

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// A spawned task that wraps a future.
///
/// Contains a boxed future and references to the task queue for re-scheduling
/// when the task is awakened. Tasks are typically created via [`Task::spawn`] and
/// should not be constructed directly in user code.
///
/// # Internals
///
/// - `future`: The wrapped future being executed
/// - `queue`: Reference to the task queue for re-scheduling
/// - `completed`: Atomic flag indicating task completion
/// - `waiters`: Wakers waiting for this task to complete
pub struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()>>>>>,
    pub(crate) queue: Arc<TaskQueue>,
    completed: AtomicBool,
    waiters: Mutex<Vec<Waker>>,
}

// Task can be safely sent across threads because:
// - Mutex<T> is Send/Sync if T is Send/Sync
// - The future is protected by a Mutex, so it's safe to share
// - AtomicBool is Send/Sync
// - Arc<TaskQueue> is Send/Sync
// Even though the future itself might not be Send, the Mutex makes it safe to share
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    /// Creates a new task wrapping the given future.
    ///
    /// Constructs a new Task by boxing the future and storing it with a reference
    /// to the task queue for re-scheduling upon awakening.
    ///
    /// # Arguments
    /// * `fut` - The future to wrap as a task (must output `()`)
    /// * `queue` - The task queue for scheduling this task
    ///
    /// # Returns
    /// An Arc-wrapped Task ready for spawning or polling
    ///
    /// # Example
    /// ```ignore
    /// let queue = Arc::new(TaskQueue::new());\n    /// let task = Task::new(async { println!(\"Hello\"); }, queue);
    /// ```
    pub(crate) fn new(fut: impl Future<Output = ()> + 'static, queue: Arc<TaskQueue>) -> Arc<Self> {
        Arc::new(Task {
            future: Mutex::new(Some(Box::pin(fut))),
            queue,
            completed: AtomicBool::new(false),
            waiters: Mutex::new(Vec::new()),
        })
    }

    /// Polls the task's future once.
    ///
    /// Attempts to make progress on the wrapped future. If the future returns [`Poll::Pending`],
    /// it is stored back for later polling. If it returns [`Poll::Ready`], the task is complete
    /// and all waiters are notified.
    ///
    /// Uses a custom waker to enable task re-scheduling when the underlying future
    /// is ready to make progress.
    ///
    /// # Panics
    /// Does not panic; errors in the future itself are caught by the future's own logic.
    ///
    /// [`Poll::Pending`]: std::task::Poll::Pending
    /// [`Poll::Ready`]: std::task::Poll::Ready
    pub fn poll(self: &Arc<Self>) {
        let waker = make_waker(self.clone());
        let mut context = Context::from_waker(&waker);

        let mut future_slot = self.future.lock().unwrap();

        if let Some(mut future) = future_slot.take() {
            match future.as_mut().poll(&mut context) {
                Poll::Pending => {
                    *future_slot = Some(future);
                }
                Poll::Ready(()) => {
                    self.completed.store(true, Ordering::SeqCst);

                    let mut waiters = self.waiters.lock().unwrap();
                    for waker in waiters.drain(..) {
                        waker.wake();
                    }
                }
            }
        }
    }

    /// Spawns a task on the current runtime context and returns a [`JoinHandle`].
    ///
    /// This function mirrors the behavior of `tokio::spawn`: it spawns a new task that
    /// runs concurrently with the current task. The returned [`JoinHandle`] can be awaited
    /// to wait for the spawned task to complete.
    ///
    /// # Requirements
    /// Must be called from within a runtime context (i.e., within an async block passed to
    /// [`Runtime::block_on`] or within another task spawned by this function).
    ///
    /// # Arguments
    /// * `future` - The future to spawn (must output `()`)
    ///
    /// # Returns
    /// A [`JoinHandle`] that can be awaited to wait for completion
    ///
    /// # Panics
    /// Panics if called outside of a runtime context.
    ///
    /// # Example
    /// ```ignore
    /// async fn example() {
    ///     let handle = Task::spawn(async {
    ///         println!("Running in background");
    ///     });
    ///     handle.await; // Wait for the task to complete
    /// }
    /// ```
    ///
    /// [`Runtime::block_on`]: crate::runtime::Runtime::block_on
    pub fn spawn<F: Future<Output = ()> + 'static>(future: F) -> JoinHandle {
        CURRENT_QUEUE.with(|current| {
            let queue = current
                .borrow()
                .as_ref()
                .expect("Task::spawn() called outside of a runtime context")
                .clone();

            let task = Task::new(future, queue.clone());
            queue.push(task.clone());

            JoinHandle { task }
        })
    }
}

/// A future that resolves when the associated task completes.
///
/// This is the return value of [`Task::spawn`]. It implements [`Future`] and can be awaited
/// to wait for the spawned task to finish execution.
///
/// # Example
/// ```ignore
/// let handle: JoinHandle = Task::spawn(async { /* ... */ });
/// handle.await; // Waits for the task to complete
/// ```
pub struct JoinHandle {
    task: Arc<Task>,
}

impl Future for JoinHandle {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task.completed.load(Ordering::SeqCst) {
            return Poll::Ready(());
        }

        let mut ws = self.task.waiters.lock().unwrap();
        ws.push(cx.waker().clone());

        Poll::Pending
    }
}

/// A helper to collect multiple [`JoinHandle`]s and await all of them at once.
///
/// This utility makes it easy to spawn multiple tasks and wait for all of them to complete
/// without explicitly looping over individual awaits.
///
/// # Example
/// ```ignore
/// let mut set = JoinSet::new();
///
/// for i in 0..5 {
///     set.push(Task::spawn(async move {
///         println!("Task {}", i);
///     }));
/// }
///
/// set.await_all().await; // Waits for all tasks
/// println!("All done");
/// ```
pub struct JoinSet {
    handles: Vec<JoinHandle>,
}

impl JoinSet {
    /// Creates a new empty JoinSet.
    ///
    /// The JoinSet starts with no handles. Push handles using [`Self::push`].
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    /// Adds a [`JoinHandle`] to the set.
    ///
    /// # Arguments
    /// * `handle` - The [`JoinHandle`] to add
    pub fn push(&mut self, handle: JoinHandle) {
        self.handles.push(handle);
    }

    /// Awaits all handles until completion, draining progressively to free memory.
    ///
    /// This method awaits each handle in order, allowing each task to complete before
    /// moving to the next one. Handles are removed from the set as they complete,
    /// freeing memory progressively.
    pub async fn await_all(&mut self) {
        for handle in self.handles.drain(..) {
            handle.await;
        }
    }
}

impl Default for JoinSet {
    fn default() -> Self {
        Self::new()
    }
}
