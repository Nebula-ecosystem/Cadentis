//! Reactor core and event handling.
//!
//! This module implements the reactor component of the runtime.
//! The reactor is responsible for:
//! - driving I/O readiness,
//! - managing timers,
//! - waking tasks when external events occur.
//!
//! It runs independently from the executor and communicates with it
//! through commands and wakers.
//!
//! Most runtime users do not interact with the reactor directly;
//! it is an internal component used by higher-level async primitives.

mod core;
mod timer;

pub(crate) mod command;
pub(crate) mod future;
pub(crate) mod io;

pub(crate) use core::{Reactor, ReactorHandle};
