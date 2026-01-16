//! TCP networking primitives.
//!
//! This module provides asynchronous TCP networking types built
//! on top of the runtime reactor and poller.
//!
//! It exposes high-level abstractions for:
//! - listening for incoming TCP connections,
//! - establishing outbound TCP connections,
//! - performing non-blocking I/O on TCP streams.
//!
//! These types integrate directly with the runtime and should be
//! used instead of blocking `std::net` sockets.
mod tcp;

pub use tcp::listener::TcpListener;
pub use tcp::stream::TcpStream;
