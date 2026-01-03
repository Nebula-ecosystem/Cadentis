use crate::reactor::core::ReactorHandle;
use crate::runtime::queue::TaskQueue;

use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Features {
    pub(crate) io_enabled: bool,
    pub(crate) fs_enabled: bool,
}

thread_local! {
    pub(crate) static CURRENT_QUEUE: RefCell<Option<Arc<TaskQueue>>> = const { RefCell::new(None) };
    pub(crate) static CURRENT_REACTOR: RefCell<Option<ReactorHandle>> = const { RefCell::new(None) };
    pub(crate) static CURRENT_FEATURES: RefCell<Option<Features>> = const { RefCell::new(None) };
}

pub(crate) fn enter_context<F, R>(
    queue: Arc<TaskQueue>,
    reactor: ReactorHandle,
    features: Features,
    function: F,
) -> R
where
    F: FnOnce() -> R,
{
    CURRENT_QUEUE.with(|current_queue| {
        CURRENT_REACTOR.with(|current_reactor| {
            CURRENT_FEATURES.with(|current_features| {
                let previous_queue = current_queue.borrow_mut().replace(queue.clone());
                let previous_reactor = current_reactor.borrow_mut().replace(reactor.clone());
                let previous_features = current_features.borrow_mut().replace(features);

                let result = function();

                *current_queue.borrow_mut() = previous_queue;
                *current_reactor.borrow_mut() = previous_reactor;
                *current_features.borrow_mut() = previous_features;

                result
            })
        })
    })
}

pub(crate) fn current_reactor_io() -> ReactorHandle {
    ensure_feature(|f| f.io_enabled, "I/O", "RuntimeBuilder::enable_io()");

    current_reactor_inner()
}

pub(crate) fn current_reactor_fs() -> ReactorHandle {
    ensure_feature(
        |f| f.fs_enabled,
        "filesystem",
        "RuntimeBuilder::enable_fs()",
    );

    current_reactor_inner()
}

fn ensure_feature(check: impl Fn(&Features) -> bool, name: &str, hint: &str) {
    CURRENT_FEATURES.with(|features| {
        let enabled = features.borrow().as_ref().map(check).unwrap_or(false);

        if !enabled {
            panic!("{} support not enabled. Use {}.", name, hint);
        }
    })
}

fn current_reactor_inner() -> ReactorHandle {
    CURRENT_REACTOR.with(|current| {
        current.borrow().clone().expect(
            "No reactor in current context. I/O operations must be called within Runtime::block_on",
        )
    })
}
