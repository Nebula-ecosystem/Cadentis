use crate::runtime::task::Task;

use std::mem;
use std::sync::Arc;
use std::task::{RawWaker, RawWakerVTable, Waker};

/// Returns the `RawWakerVTable` for a task of type `T`.
///
/// The vtable defines how the executor interacts with the task when:
/// - cloning the waker,
/// - waking the task,
/// - waking by reference,
/// - dropping the waker.
///
/// # Safety
///
/// All functions in the vtable must uphold the invariants required
/// by [`RawWaker`], in particular:
/// - reference counts must be correctly managed,
/// - the task must remain valid for the lifetime of the waker.
fn vtable<T: Send + 'static>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_raw::<T>,
        wake_raw::<T>,
        wake_by_ref_raw::<T>,
        drop_raw::<T>,
    )
}

/// Creates a [`Waker`] associated with a runtime task.
///
/// The returned waker will reschedule the task when woken.
///
/// # Safety
///
/// This function relies on a custom `RawWaker` implementation backed
/// by an `Arc<Task<T>>`. The pointer stored inside the `RawWaker`
/// must originate from `Arc::into_raw` and follow proper reference
/// counting semantics.
///
/// This function is safe to call as long as the `Task` correctly
/// implements the wake logic.
pub(crate) fn make_waker<T: Send + 'static>(task: Arc<Task<T>>) -> Waker {
    unsafe {
        Waker::from_raw(RawWaker::new(
            Arc::into_raw(task) as *const (),
            vtable::<T>(),
        ))
    }
}

/// Clones the raw waker.
///
/// This increments the reference count of the underlying `Arc<Task<T>>`
/// and returns a new `RawWaker` pointing to the same task.
fn clone_raw<T: Send + 'static>(ptr: *const ()) -> RawWaker {
    let arc = unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
    let cloned = arc.clone();
    mem::forget(arc);

    RawWaker::new(Arc::into_raw(cloned) as *const (), vtable::<T>())
}

/// Wakes the task and consumes the waker.
///
/// This transfers ownership of the `Arc<Task<T>>` and calls
/// [`Task::wake`], potentially scheduling the task for execution.
fn wake_raw<T: Send + 'static>(ptr: *const ()) {
    let arc = unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
    arc.wake();
}

/// Wakes the task without consuming the waker.
///
/// The underlying `Arc<Task<T>>` is cloned to preserve the original
/// reference count.
fn wake_by_ref_raw<T: Send + 'static>(ptr: *const ()) {
    let arc = unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
    arc.clone().wake();
    mem::forget(arc);
}

/// Drops the raw waker.
///
/// This decrements the reference count of the underlying `Arc<Task<T>>`.
/// No other action is performed.
fn drop_raw<T: Send + 'static>(ptr: *const ()) {
    unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
}
