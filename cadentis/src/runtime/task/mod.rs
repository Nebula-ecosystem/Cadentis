//! Asynchronous task primitives.
//!
//! This module defines the core abstractions used by the runtime to
//! represent, schedule, and execute asynchronous tasks.
//!
//! It includes:
//! - **Task state management**: Atomic state transitions (IDLE, RUNNING, etc.).
//! - **Task & Runnable**: The core unit of work executed by the scheduler.
//! - **JoinHandle**: A handle to await the result of a single spawned task.
//! - **JoinSet**: A collection of tasks that allows awaiting their completion
//!   collectively or managing their lifecycle (e.g., mass cancellation).
//!
//! Most users will interact with this module through [`spawn`] to launch
//! individual tasks or [`JoinSet`] to manage multiple concurrent tasks.

pub(crate) mod handle;
pub(crate) mod set;
pub(crate) mod state;
pub(crate) mod waker;

pub(crate) use core::{Runnable, Task};
pub(crate) use handle::JoinHandle;

pub mod core;

pub use core::spawn;
pub use set::JoinSet;
