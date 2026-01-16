//! Work-stealing scheduler components.
//!
//! This module implements the core data structures used by the
//! executor to distribute tasks across worker threads using
//! a work-stealing strategy.
//!
//! It consists of:
//! - [`injector`]: a global queue for newly spawned tasks,
//! - [`queue`]: per-worker local queues used for fast local execution
//!   and task stealing.
//!
//! This design minimizes contention while maintaining good load
//! balancing across threads.

pub(crate) mod injector;
pub(crate) mod queue;
