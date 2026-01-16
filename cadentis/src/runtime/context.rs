use crate::reactor::ReactorHandle;
use crate::runtime::work_stealing::injector::InjectorHandle;
use crate::runtime::work_stealing::queue::LocalQueue;

use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// Thread-local handle to the current reactor.
    ///
    /// This is set when entering the runtime context and allows
    /// runtime components (timers, I/O, etc.) to access the reactor
    /// without explicit parameter passing.
    pub(crate) static CURRENT_REACTOR: RefCell<Option<ReactorHandle>> =
        const { RefCell::new(None) };

    /// Thread-local handle to the global injector queue.
    ///
    /// Used by worker threads to push tasks into the global work-stealing
    /// queue.
    pub(crate) static CURRENT_INJECTOR: RefCell<Option<InjectorHandle>> =
        const { RefCell::new(None) };

    /// Thread-local identifier of the current worker thread.
    pub(crate) static CURRENT_WORKER_ID: RefCell<Option<usize>> =
        const { RefCell::new(None) };

    /// Thread-local references to all local worker queues.
    ///
    /// This allows work stealing between workers without global
    /// synchronization.
    pub(crate) static CURRENT_LOCALS: RefCell<Option<Arc<Vec<Arc<LocalQueue>>>>> =
        const { RefCell::new(None) };
}

/// Enters the runtime execution context for the current thread.
///
/// This function temporarily installs thread-local runtime state
/// (reactor and injector handles) for the duration of the closure `f`.
/// After the closure completes, the previous context is restored.
///
/// This mechanism allows deeply nested runtime components to access
/// shared execution state without passing handles through every API.
///
/// # Arguments
///
/// * `reactor` - Handle to the runtime reactor.
/// * `injector` - Handle to the global task injector.
/// * `f` - Closure executed inside the runtime context.
///
/// # Returns
///
/// Returns the result of the closure `f`.
///
/// # Panics
///
/// Panics if thread-local access fails (should never happen).
pub(crate) fn enter_context<R>(
    reactor: ReactorHandle,
    injector: InjectorHandle,
    f: impl FnOnce() -> R,
) -> R {
    CURRENT_REACTOR.with(|r| {
        CURRENT_INJECTOR.with(|i| {
            let prev_r = r.replace(Some(reactor));
            let prev_i = i.replace(Some(injector));

            let out = f();

            i.replace(prev_i);
            r.replace(prev_r);

            out
        })
    })
}
