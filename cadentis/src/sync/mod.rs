//! Synchronization primitives for Cadentis.
//!
//! This module provides async-aware synchronization tools for the runtime.
//! These primitives are designed to work seamlessly with the Cadentis task scheduler
//! and reactor, enabling safe sharing of state between tasks without blocking threads.
//!
//! The current primitives include:
//! - [`Mutex`] — an asynchronous mutual exclusion primitive.
//!
//! ## Design notes
//!
//! - The primitives are lightweight and do not spawn threads themselves.
//! - Tasks that cannot immediately acquire a lock are suspended and woken
//!   when the resource becomes available.
//! - Mutexes are safe to share between threads and tasks using `Arc`.
//!
//! Most runtime users will use these primitives indirectly when sharing
//! state between tasks; advanced users can use them directly for custom data structures.

mod mutex;

pub use mutex::Mutex;
