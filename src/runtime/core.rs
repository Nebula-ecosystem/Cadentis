use std::sync::mpsc;

use super::executor::core::Executor;
use crate::reactor::command::Command;
use crate::reactor::{Reactor, ReactorHandle};

pub struct Runtime {
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

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.executor.spawn(future);
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (transmitter, receiver) = mpsc::channel();

        self.spawn(async move {
            let result = future.await;
            let _ = transmitter.send(result);
        });

        receiver.recv().expect("block_on failed")
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.executor.shutdown();

        let _ = self.reactor_handle.send(Command::Shutdown);
        let _ = self.reactor_handle.send(Command::Wake);

        self.executor.join();
    }
}
