/// An I/O event reported by the poller.
///
/// An `Event` represents readiness information for a registered
/// file descriptor. It is produced by the poller and consumed
/// by the reactor to wake the appropriate tasks.
///
/// The event indicates whether the file descriptor is readable,
/// writable, or both.
pub(crate) struct Event {
    /// Token associated with the registered file descriptor.
    ///
    /// This token is used to identify the I/O entry inside the reactor.
    pub(crate) token: usize,

    /// Indicates that the file descriptor is readable.
    pub(crate) readable: bool,

    /// Indicates that the file descriptor is writable.
    pub(crate) writable: bool,
}
