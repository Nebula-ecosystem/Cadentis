use crate::runtime::task::Task;

use std::sync::Arc;
use std::task::{RawWaker, RawWakerVTable, Waker};

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone_raw, wake_raw, wake_by_ref_raw, drop_raw);

pub(crate) fn make_waker(task: Arc<Task>) -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(Arc::into_raw(task) as *const (), &VTABLE)) }
}

fn clone_raw(ptr: *const ()) -> RawWaker {
    let arc = unsafe { Arc::<Task>::from_raw(ptr as *const Task) };
    let cloned = arc.clone();
    std::mem::forget(arc);

    RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
}

fn wake_raw(ptr: *const ()) {
    let arc = unsafe { Arc::<Task>::from_raw(ptr as *const Task) };
    arc.wake();
}

fn wake_by_ref_raw(ptr: *const ()) {
    let arc = unsafe { Arc::<Task>::from_raw(ptr as *const Task) };
    arc.wake();
}

fn drop_raw(ptr: *const ()) {
    unsafe { Arc::<Task>::from_raw(ptr as *const Task) };
}
