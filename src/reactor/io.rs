//! Connection state and data structures for I/O management.
//!
//! This module provides the core types for tracking connection state and buffering
//! data for non-blocking I/O operations.

/// Represents the current state of a client connection.
///
/// A connection alternates between reading from the client and writing back to it.
pub(crate) enum ConnectionState {
    /// The connection is ready to read data from the client.
    Reading,

    /// The connection has data to write back to the client.
    Writing,
}

/// Represents an active client connection with buffered output.
///
/// This structure maintains the connection's state and a buffer of data
/// that needs to be written back to the client.
pub(crate) struct Connection {
    /// The current state of the connection (reading or writing).
    pub(crate) state: ConnectionState,

    /// Buffer containing data to be written to the client.
    pub(crate) out: Vec<u8>,
}

impl Connection {
    /// Creates a new connection in the Reading state.
    ///
    /// The connection starts in the Reading state with an empty output buffer.
    pub(crate) fn new() -> Self {
        Self {
            state: ConnectionState::Reading,
            out: Vec::new(),
        }
    }
}
