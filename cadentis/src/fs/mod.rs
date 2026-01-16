//! Asynchronous filesystem primitives.
//!
//! This module provides non-blocking filesystem abstractions built
//! on top of the runtime reactor.
//!
//! It exposes high-level types for:
//! - working with directories ([`Dir`]),
//! - reading from and writing to files ([`File`]).
//!
//! These types integrate with the runtime and avoid blocking
//! the executor threads.

mod dir;
mod file;

pub use dir::Dir;
pub use file::File;
