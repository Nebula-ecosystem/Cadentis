//! Instrumented time utilities.
//!
//! This module provides time-related asynchronous utilities that
//! integrate with the runtime reactor.
//!
//! It includes:
//! - [`sleep`] for scheduling timers,
//! - [`timeout`] for bounding future execution time,
//! - [`instrumented`] for wrapping and observing async execution.

mod instrumented;
mod sleep;
mod timeout;

#[doc(inline)]
pub use instrumented::instrumented;

#[doc(inline)]
pub use sleep::sleep;

#[doc(inline)]
pub use timeout::timeout;
