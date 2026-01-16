//! Retry utilities for asynchronous operations.
//!
//! This module provides helpers for retrying fallible asynchronous
//! operations with optional delays between attempts.
//!
//! The main entry point is [`retry`], which creates a future that
//! retries an operation produced by a factory closure until it
//! succeeds or the retry limit is reached.

mod retry;

#[doc(inline)]
pub use retry::retry;
