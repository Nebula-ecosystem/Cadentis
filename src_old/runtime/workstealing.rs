impl Worker {
    pub fn run(&self) {
        CURRENT_INJECTOR.with(|cell| {
            *cell.borrow_mut() = Some(self.injector.clone());
        });
        CURRENT_REACTOR.with(|cell| {
            *cell.borrow_mut() = Some(self.reactor.clone());
        });
        CURRENT_FEATURES.with(|cell| {
            *cell.borrow_mut() = Some(self.features);
        });

        loop {
            if let Some(task) = self.try_steal() {
                task.poll();
            } else {
                let queue = self.injector.queue.lock().unwrap();

                if queue.is_empty() && !self.injector.is_shutdown() {
                    let _ = self
                        .injector
                        .condvar
                        .wait_timeout(queue, std::time::Duration::from_millis(10));
                }
            }
        }
    }

    fn try_steal(&self) -> Option<Arc<dyn Runnable>> {
        let len = self.locals.len();

        for i in 0..len {
            let victim = (self.id + i + 1) % len;

            if let Some(task) = self.locals[victim].steal() {
                return Some(task);
            }
        }
        None
    }
}
