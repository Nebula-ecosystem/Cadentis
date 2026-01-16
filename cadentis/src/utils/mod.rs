//! Utilities for memory-efficient data structures.
//!
//! This module provides low-level utilities used internally by the runtime.
//! In particular, it exposes a [`Slab`] allocator used for fast indexed
//! storage with reuse of freed slots.

mod slab;

pub(crate) use slab::Slab;
