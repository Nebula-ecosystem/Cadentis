//! # Cadentis
//!
//! **Cadentis** is a lightweight async runtime for Rust, designed as the dedicated task
//! orchestration layer for the **Nebula** ecosystem.
//!
//! Unlike general-purpose runtimes like Tokio or async-std, Cadentis focuses on providing
//! only the essential primitives required by the Nebula platform. It features a work-stealing
//! multi-threaded executor, non-blocking I/O powered by kqueue (macOS), and a minimal API
//! surface that keeps the runtime lean and efficient.
//!
//! Cadentis is built from the ground up with simplicity and performance in mind, offering:
//!
//! - A **work-stealing scheduler** that distributes tasks efficiently across worker threads
//! - **Async file and directory operations** for non-blocking filesystem access
//! - **Async TCP networking** with listener and stream abstractions
//! - **Timer primitives** including sleep, timeout, and intervals
//! - **Ergonomic macros** like `#[cadentis::main]`, `#[cadentis::test]`, `join!`, and `select!`
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cadentis::time::sleep;
//! use cadentis::task;
//! use std::time::Duration;
//!
//! #[cadentis::main]
//! async fn main() {
//!     // Spawn a background task
//!     let handle = task::spawn(async {
//!         sleep(Duration::from_millis(100)).await;
//!         println!("Task completed!");
//!     });
//!
//!     // Wait for the task to finish
//!     handle.await;
//! }
//! ```
//!
//! ## Modules
//!
//! - [`fs`] — Async file and directory operations
//! - [`net`] — Async networking (TCP listener/stream)
//! - [`time`] — Timers, sleep, timeout, and intervals
//! - [`tools`] — Utilities like retry mechanisms
//!
//! ## Getting Started
//!
//! Add Cadentis to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cadentis = { git = "https://github.com/Nebula-ecosystem/Cadentis", package = "cadentis" }
//! ```

mod reactor;
mod runtime;
mod utils;

pub mod fs;
pub mod net;
pub mod time;
pub mod tools;

pub use runtime::builder::RuntimeBuilder;
pub use runtime::task;
pub use runtime::yield_now::yield_now;

pub use cadentis_macros::*;
