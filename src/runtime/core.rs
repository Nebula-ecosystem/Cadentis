use crate::executor::Executor;
use crate::executor::command::Command as ExecutorCommand;
use crate::reactor::Reactor;
use crate::reactor::command::Command as ReactorCommand;
use crate::runtime::workstealing::Injector;

use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;

pub struct Runtime {
    executor_transmitter: Sender<ExecutorCommand>,
    reactor_transmitter: Sender<ReactorCommand>,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let injector = Arc::new(Injector::new());
        let (reactor, reactor_transmitter) = Reactor::new();
        let (executor, executor_transmitter) = Executor::new(injector, reactor_transmitter.clone());

        thread::spawn(move || reactor.run());
        thread::spawn(move || executor.run());

        Self {
            executor_transmitter,
            reactor_transmitter,
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.reactor_transmitter.send(ReactorCommand::Shutdown);
        self.executor_transmitter.send(ExecutorCommand::Shutdown);
    }
}
