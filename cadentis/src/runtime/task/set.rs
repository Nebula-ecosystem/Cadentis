use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::task;

/// A collection of tasks that allows awaiting their completion
/// collectively or managing their lifecycle as a group.
///
/// `JoinSet` manages a dynamic number of asynchronous tasks. It is
/// especially useful for scenarios like:
/// - Processing a list of requests and acting on the first few that finish.
/// - Ensuring all background tasks are cancelled if a parent process stops.
/// - Limiting concurrency by spawning and joining tasks in a loop.
pub struct JoinSet {
    /// Internal collection of task handles stored as pinned trait objects.
    /// Handles are stored as `dyn SetHandle` to allow the set to manage tasks
    /// returning different types `T` internally.
    pub(crate) handles: Vec<Pin<Box<dyn SetHandle>>>,
}

impl JoinSet {
    /// Creates a new, empty `JoinSet`.
    ///
    /// No tasks are spawned until [`spawn`] is called.
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    /// Spawns a new task into the set.
    ///
    /// The task is immediately scheduled on the runtime. A handle to the task
    /// is added to the internal collection.
    ///
    /// # Arguments
    /// * `fut` - The asynchronous computation to run.
    pub fn spawn<F, T>(&mut self, fut: F)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let handle = task::spawn(fut);
        self.handles.push(Box::pin(handle));
    }

    /// Returns the number of tasks currently managed by the set.
    ///
    /// This includes tasks that are currently running and tasks that
    /// have finished but have not yet been "joined" via [`join_next`].
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Returns `true` if the set contains no tasks.
    ///
    /// This is useful for checking if all spawned tasks have been
    /// processed or if the set is ready to be reused.
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Waits for the next task in the set to complete.
    ///
    /// This method polls all managed tasks. When one completes, it is removed
    /// from the set and the method returns `Some(())`.
    ///
    /// # Returns
    /// - `Some(())` if a task completed.
    /// - `None` if the set was empty.
    pub async fn join_next(&mut self) -> Option<()> {
        if self.handles.is_empty() {
            return None;
        }

        poll_fn(|cx| {
            let mut i = 0;

            while i < self.handles.len() {
                match self.handles[i].as_mut().poll_completed(cx) {
                    Poll::Ready(()) => {
                        // O(1) removal by swapping with the last element.
                        // Order is not preserved, but efficiency is maximized.
                        self.handles.swap_remove(i);
                        return Poll::Ready(Some(()));
                    }
                    Poll::Pending => {
                        i += 1;
                    }
                }
            }
            Poll::Pending
        })
        .await
    }

    /// Aborts all tasks currently managed by the set.
    ///
    /// This signals every task to stop execution. The internal collection
    /// is cleared immediately. Any results from tasks that had not yet
    /// been joined are discarded.
    pub fn abort_all(&mut self) {
        for handle in &self.handles {
            handle.abort();
        }
        self.handles.clear();
    }

    /// Waits for `n` tasks to complete, then aborts all remaining tasks.
    ///
    /// This is a cooperative "quota" mechanism. Once the first `n` tasks
    /// finish, the rest of the set is cleaned up.
    ///
    /// # Returns
    /// - `Ok(())` if `n` tasks successfully completed.
    /// - `Err(())` if the set was exhausted before `n` tasks could finish.
    ///
    /// # Note
    /// Even if an error is returned, all remaining tasks are aborted.
    pub async fn race_n(&mut self, n: usize) -> Result<(), ()> {
        if n > self.handles.len() {
            return Err(());
        }

        for _ in 0..n {
            if self.join_next().await.is_none() {
                self.abort_all();
                return Err(());
            }
        }

        self.abort_all();
        Ok(())
    }

    /// Waits for the first task to complete and aborts all others.
    ///
    /// This is a convenience method for `race_n(1)`. Useful for
    /// speculative execution (first responder wins).
    pub async fn race(&mut self) -> Result<(), ()> {
        self.race_n(1).await
    }

    /// Waits for all tasks in the set to complete.
    ///
    /// This will drive all managed futures to completion and clear the set.
    pub async fn join_all(&mut self) {
        while self.join_next().await.is_some() {}
    }
}

impl Default for JoinSet {
    /// Returns an empty [`JoinSet`].
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for JoinSet {
    /// Automatically aborts all tasks when the [`JoinSet`] is dropped.
    ///
    /// This ensures that tasks do not continue to run in the background
    /// (leaking resources) once the manager is no longer in scope.
    fn drop(&mut self) {
        self.abort_all();
    }
}

/// Internal trait used to abstract over different JoinHandle result types.
///
/// This allows a `JoinSet` to manage a heterogeneous collection of tasks
/// returning different types while providing a uniform interface for
/// polling and cancellation.
pub(crate) trait SetHandle: Send {
    /// Polled by the `JoinSet` to check for task completion.
    fn poll_completed(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;

    /// Signals the task to stop execution.
    fn abort(&self);
}
