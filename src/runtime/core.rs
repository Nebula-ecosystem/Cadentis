use super::executor::core::Executor;
use crate::reactor::command::Command;
use crate::reactor::{Reactor, ReactorHandle};

pub(crate) struct Runtime {
    executor: Executor,
    reactor_handle: ReactorHandle,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let reactor_handle = Reactor::start();
        let executor = Executor::new(reactor_handle.clone(), 2); // To change

        Self {
            executor,
            reactor_handle,
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.executor.shutdown();
        self.reactor_handle.send(Command::Shutdown);
        self.executor.join();
    }
}
