//! Asynchronous task primitives.
//!
//! This module defines the core abstractions used by the runtime to
//! represent, schedule, and execute asynchronous tasks.
//!
//! It includes:
//! - task state management,
//! - custom waker integration,
//! - join handles for awaiting task completion,
//! - the core task and runnable abstractions.
//!
//! Most users will interact with this module through [`spawn`] and
//! //! Join handles for spawned tasks, while the lower-level components
//! are used internall by the executor.

pub(crate) mod handle;
pub(crate) mod state;
pub(crate) mod waker;

pub(crate) use core::{Runnable, Task};
pub(crate) use handle::JoinHandle;

pub mod core;

pub use core::spawn;
