use std::cell::UnsafeCell;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Mutex as Mutex_std;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

/// An asynchronous mutex.
///
/// `Mutex<T>` provides mutual exclusion for async tasks. Unlike
/// a standard `std::sync::Mutex`, this mutex does not block threads
/// when waiting; tasks that cannot acquire the lock are suspended
/// and woken up when the mutex becomes available.
///
/// This is the async equivalent of a traditional mutex.
pub struct Mutex<T> {
    /// Indicates whether the mutex is currently locked.
    ///
    /// `AtomicBool` is used for lock-free checking and acquisition.
    locked: AtomicBool,

    /// List of wakers for tasks waiting to acquire the mutex.
    ///
    /// Protected by a standard blocking `Mutex` because manipulating
    /// the waiters list is fast and infrequent.
    waiters: Mutex_std<Vec<Waker>>,

    /// The underlying data protected by the mutex.
    ///
    /// UnsafeCell allows mutable access through shared references,
    /// which is safe because we guarantee mutual exclusion.
    data: UnsafeCell<T>,
}

// Safety: `Mutex<T>` can be sent across threads if `T` is Send.
unsafe impl<T: Send> Send for Mutex<T> {}
// Safety: `Mutex<T>` can be shared across threads if `T` is Send,
// because access is synchronized using `AtomicBool` and waiters queue.
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex wrapping the given value.
    ///
    /// # Example
    /// ```
    /// let mutex = Mutex::new(42);
    /// ```
    ///
    /// The mutex is initially unlocked, and no waiters are present.
    pub fn new(value: T) -> Mutex<T> {
        Self {
            // Indicates whether the mutex is currently locked.
            locked: AtomicBool::new(false),

            // List of tasks waiting to acquire the mutex.
            // Protected by a standard Mutex to ensure safe concurrent access.
            waiters: Mutex_std::new(Vec::new()),

            // The data protected by the mutex.
            // UnsafeCell allows mutable access even through a shared reference,
            // which is required for interior mutability in async contexts.
            data: UnsafeCell::new(value),
        }
    }

    /// Returns a future that will resolve to a guard when the mutex is acquired.
    ///
    /// This does **not block the thread**. Instead, the task is suspended until
    /// the mutex becomes available.
    ///
    /// # Example
    /// ```rust
    /// let guard = mutex.lock().await;
    /// // The protected value can now be accessed via `*guard`.
    /// ```
    pub fn lock(&self) -> LockFuture<'_, T> {
        LockFuture { mutex: self }
    }
}

/// Future returned by `Mutex::lock`.
///
/// The future resolves to a `MutexGuard` once the lock is acquired.
pub struct LockFuture<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    /// Polls the future to attempt acquiring the mutex.
    ///
    /// If the mutex is free, the future resolves immediately.
    /// If the mutex is locked, the current task is registered
    /// in the waiters queue and the future returns `Poll::Pending`.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Attempt to acquire the lock atomically.
        if !self.mutex.locked.swap(true, Ordering::Acquire) {
            // Lock acquired immediately.
            return Poll::Ready(MutexGuard { mutex: self.mutex });
        }

        // Lock is already held, register the task to be woken later.
        let mut waiters = self.mutex.waiters.lock().unwrap();
        waiters.push(cx.waker().clone());

        Poll::Pending
    }
}

/// Guard returned by `Mutex::lock`.
///
/// Releases the mutex when dropped.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    /// Unlocks the mutex and wakes one waiting task (if any).
    fn drop(&mut self) {
        // Release the lock.
        self.mutex.locked.store(false, Ordering::Release);

        // Wake the next waiting task.
        if let Some(waker) = self.mutex.waiters.lock().unwrap().pop() {
            waker.wake();
        }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    /// Provides immutable access to the protected data.
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    /// Provides mutable access to the protected data.
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}
