struct FutureSlot<F> {
    future: UnsafeCell<Pin<Box<dyn Future<Output = T> + Send>>>,
    polling: AtomicBool,
}
