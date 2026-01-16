use std::cmp::Ordering;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::task::Waker;
use std::time::Instant;

/// An entry in the reactor timer queue.
///
/// `TimerEntry` represents a scheduled wake-up at a specific
/// deadline. It is typically stored inside a priority queue
/// (e.g. a binary heap) ordered by deadline.
///
/// The entry may be cancelled before it fires.
pub(crate) struct TimerEntry {
    /// The time at which the timer should fire.
    pub(crate) deadline: Instant,

    /// Waker to notify when the deadline is reached.
    pub(crate) waker: Waker,

    /// Cancellation flag shared with the associated sleep future.
    pub(crate) cancelled: Arc<AtomicBool>,
}

impl Eq for TimerEntry {}

impl PartialEq for TimerEntry {
    /// Two timer entries are equal if their deadlines are equal.
    fn eq(&self, other: &Self) -> bool {
        self.deadline.eq(&other.deadline)
    }
}

impl Ord for TimerEntry {
    /// Orders timer entries by deadline.
    ///
    /// Note that the comparison is **reversed** so that a
    /// `BinaryHeap<TimerEntry>` behaves as a min-heap,
    /// where the earliest deadline is popped first.
    fn cmp(&self, other: &Self) -> Ordering {
        other.deadline.cmp(&self.deadline)
    }
}

impl PartialOrd for TimerEntry {
    /// Partial ordering consistent with [`Ord`].
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
