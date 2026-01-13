use crate::reactor::ReactorHandle;
use crate::runtime::work_stealing::injector::InjectorHandle;
use crate::runtime::work_stealing::queue::LocalQueue;

use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    pub(crate) static CURRENT_REACTOR: RefCell<Option<ReactorHandle>> = const { RefCell::new(None) };
    pub(crate) static CURRENT_INJECTOR: RefCell<Option<InjectorHandle>> = const { RefCell::new(None) };
    pub(crate) static CURRENT_WORKER_ID: RefCell<Option<usize>> = const { RefCell::new(None) };
    pub(crate) static CURRENT_LOCALS: RefCell<Option<Arc<Vec<Arc<LocalQueue>>>>> =
        const { RefCell::new(None) };
}

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
