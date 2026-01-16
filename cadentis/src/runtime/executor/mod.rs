//! Task executor implementation.
//!
//! This module contains the core components responsible for executing
//! asynchronous tasks within the runtime.
//!
//! It is composed of:
//! - [`core`]: the main executor logic and lifecycle management,
//! - [`worker`]: worker threads that run tasks using work-stealing.
//!
//! Together, these components implement a scalable, multi-threaded
//! executor integrated with the runtime reactor.

pub(crate) mod core;
pub(crate) mod worker;
