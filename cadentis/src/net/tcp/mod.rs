//! TCP networking implementation.
//!
//! This module contains the concrete TCP types built on top of the
//! runtime reactor and poller.
//!
//! It is split into:
//! - [`listener`]: accepting incoming TCP connections,
//! - [`stream`]: asynchronous TCP streams with buffered I/O.
//!
//! These types provide a non-blocking, async alternative to
//! `std::net::TcpListener` and `std::net::TcpStream`.

pub mod listener;
pub mod stream;
