/// Task is idle and not scheduled.
///
/// The task exists but is not currently queued or running.
pub(crate) const IDLE: usize = 0;

/// Task is queued for execution.
///
/// The task has been scheduled and is waiting in a run queue.
pub(crate) const QUEUED: usize = 1;

/// Task is currently being executed by a worker.
///
/// At most one worker may observe this state at a time.
pub(crate) const RUNNING: usize = 2;

/// Task has completed execution.
///
/// The future has returned `Poll::Ready` and will not be polled again.
pub(crate) const COMPLETED: usize = 3;

/// Task has been notified while running.
///
/// This state indicates that the task was woken while already
/// executing and should be re-queued once execution finishes.
pub(crate) const NOTIFIED: usize = 4;

/// Task has been cancelled.
///
/// The task was aborted before completion. Any further polling
/// will result in an immediate return or panic depending on the handle.
pub(crate) const CANCELLED: usize = 5;
