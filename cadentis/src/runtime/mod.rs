//! Core runtime components.
//!
//! This module contains the fundamental building blocks of the runtime,
//! including task execution, scheduling, and cooperative yielding.
//!
//! It is responsible for:
//! - executing asynchronous tasks,
//! - managing task queues and work stealing,
//! - providing runtime context and task-local facilities,
//! - enabling cooperative multitasking via yielding.
//!
//! Most users will interact with higher-level APIs built on top of
//! these components rather than using this module directly.

mod core;
mod executor;
mod work_stealing;

pub(crate) mod builder;
pub(crate) mod context;
pub(crate) mod yield_now;

pub mod task;

use core::Runtime;
