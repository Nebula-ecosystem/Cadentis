use crate::runtime::task::Task;

use std::mem;
use std::sync::Arc;
use std::task::{RawWaker, RawWakerVTable, Waker};

fn vtable<T: Send + 'static>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_raw::<T>,
        wake_raw::<T>,
        wake_by_ref_raw::<T>,
        drop_raw::<T>,
    )
}

pub(crate) fn make_waker<T: Send + 'static>(task: Arc<Task<T>>) -> Waker {
    unsafe {
        Waker::from_raw(RawWaker::new(
            Arc::into_raw(task) as *const (),
            vtable::<T>(),
        ))
    }
}

fn clone_raw<T: Send + 'static>(ptr: *const ()) -> RawWaker {
    let arc = unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
    let cloned = arc.clone();
    mem::forget(arc);

    RawWaker::new(Arc::into_raw(cloned) as *const (), vtable::<T>())
}

fn wake_raw<T: Send + 'static>(ptr: *const ()) {
    let arc = unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
    arc.wake();
}

fn wake_by_ref_raw<T: Send + 'static>(ptr: *const ()) {
    let arc = unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
    arc.clone().wake();
    mem::forget(arc);
}

fn drop_raw<T: Send + 'static>(ptr: *const ()) {
    unsafe { Arc::<Task<T>>::from_raw(ptr as *const Task<T>) };
}
