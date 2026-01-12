use crate::reactor::ReactorHandle;

use std::cell::RefCell;

thread_local! {
    pub(crate) static CURRENT_REACTOR: RefCell<Option<ReactorHandle>> = const { RefCell::new(None) };
}

pub(crate) fn enter_context<R>(reactor: ReactorHandle, f: impl FnOnce() -> R) -> R {
    CURRENT_REACTOR.with(|cell| {
        let prev = cell.replace(Some(reactor));
        let r = f();

        cell.replace(prev);

        r
    })
}
