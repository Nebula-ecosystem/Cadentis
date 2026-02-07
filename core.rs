use nucleus::poll::{Event, Poller};

use super::command::Command;
use super::io::IoEntry;
use super::timer::TimerEntry;
use crate::reactor::io::Waiting;
use crate::utils::Slab;

use std::collections::BinaryHeap;
use std::io;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::SendError;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Instant;

/// The reactor.
///
/// The reactor runs on a dedicated thread and is responsible for:
/// - polling OS I/O readiness events,
/// - managing timers,
/// - waking tasks when I/O or timers become ready,
/// - coordinating with the executor via commands.
///
/// It communicates with the rest of the runtime through
/// [`Command`] messages sent over a channel.
pub(crate) struct Reactor {
    /// Channel receiving commands from executor threads.
    receiver: Receiver<Command>,

    /// Platform-specific poller (epoll, kqueue, etc.).
    poller: Poller,

    /// Buffer used to collect I/O events from the poller.
    events: Vec<Event>,

    /// Min-heap of pending timers ordered by deadline.
    timers: BinaryHeap<TimerEntry>,

    /// Slab storing active I/O entries indexed by poller tokens.
    io: Slab<IoEntry>,
}

/// A handle used to communicate with the reactor thread.
///
/// Cloning this handle allows multiple threads to:
/// - register and deregister I/O,
/// - schedule timers,
/// - wake the reactor when new commands arrive.
#[derive(Clone)]
pub(crate) struct ReactorHandle {
    /// Sender side of the command channel.
    sender: Sender<Command>,

    /// Waker used to interrupt the poller.
    waker: Arc<Waker>,
}

impl ReactorHandle {
    /// Sends a command to the reactor and wakes it.
    pub(crate) fn send(&self, cmd: Command) -> Result<(), SendError<Command>> {
        let result = self.sender.send(cmd);
        self.waker.wake();
        result
    }
}

impl Reactor {
    /// Creates a new reactor instance.
    fn new(receiver: Receiver<Command>, poller: Poller) -> Self {
        let events = Vec::with_capacity(64);
        let timers = BinaryHeap::new();
        let io = Slab::new(64);

        Self {
            receiver,
            poller,
            events,
            timers,
            io,
        }
    }

    /// Starts the reactor thread and returns a handle to it.
    pub(crate) fn start() -> ReactorHandle {
        let (sender, rx) = channel();
        let poller = Poller::new();
        let waker = poller.waker();

        thread::spawn(move || {
            let mut reactor = Reactor::new(rx, poller);
            reactor.run().unwrap();
        });

        ReactorHandle { sender, waker }
    }

    /// Main reactor event loop.
    ///
    /// The loop performs the following steps:
    /// 1. Handle I/O events from the previous poll
    /// 2. Process pending commands
    /// 3. Poll the OS for new events (with timer-based timeout)
    /// 4. Fire expired timers
    fn run(&mut self) -> io::Result<()> {
        loop {
            // Handle previously collected I/O events
            let events: Vec<Event> = self.events.drain(..).collect();
            for event in events {
                self.handle_event(event);
            }

            // Process incoming commands
            while let Ok(cmd) = self.receiver.try_recv() {
                match cmd {
                    Command::Register {
                        fd,
                        interest,
                        entry,
                    } => {
                        let token = self.io.insert(entry);
                        self.poller.register(fd, token, interest);
                    }
                    Command::Deregister { fd } => {
                        self.poller.deregister(fd);
                    }
                    Command::SetTimer {
                        deadline,
                        waker,
                        cancelled,
                    } => {
                        self.timers.push(TimerEntry {
                            deadline,
                            waker,
                            cancelled,
                        });
                    }
                    Command::Shutdown => {
                        return Ok(());
                    }
                }
            }

            // Compute poll timeout from next timer
            let timeout = self
                .timers
                .peek()
                .map(|t| t.deadline.saturating_duration_since(Instant::now()));

            // Poll for I/O events
            self.poller.poll(&mut self.events, timeout)?;

            // Fire expired timers
            let now = Instant::now();
            while let Some(timer) = self.timers.peek() {
                if timer.deadline > now {
                    break;
                }

                let timer = self.timers.pop().unwrap();

                if timer.cancelled.load(Ordering::Acquire) {
                    continue;
                }

                timer.waker.wake();
            }
        }
    }

    /// Handles a single I/O event from the poller.
    fn handle_event(&mut self, event: Event) {
        let mut should_close = false;
        let mut fd = None;
        let mut new_interest = None;

        {
            let entry = self.io.get_mut(event.token);

            match entry {
                // One-shot waiter
                IoEntry::Waiting(Waiting { waker, interest }) => {
                    let mut woke = false;

                    if event.readable && interest.read {
                        waker.wake_by_ref();
                        woke = true;
                    }

                    if event.writable && interest.write {
                        waker.wake_by_ref();
                        woke = true;
                    }

                    if woke {
                        self.io.remove(event.token);
                    }
                }

                // Buffered stream
                IoEntry::Stream(stream) => {
                    let mut stream = stream.lock().unwrap();
                    fd = Some(stream.fd);

                    if event.readable {
                        if handle_read(stream.fd, &mut stream.in_buffer) {
                            should_close = true;
                        } else {
                            stream.read_waiters.drain(..).for_each(|w| w.wake());
                        }
                    }

                    if !should_close && event.writable {
                        if handle_write(stream.fd, &mut stream.out_buffer) {
                            should_close = true;
                        } else if stream.out_buffer.is_empty() {
                            stream.write_waiters.drain(..).for_each(|w| w.wake());
                        }
                    }

                    new_interest = Some(stream.interest());
                }
            }
        }

        if let Some(fd) = fd {
            if should_close {
                self.cleanup(event.token, fd);
            } else if let Some(ni) = new_interest {
                self.poller.reregister(fd, event.token, ni);
            }
        }
    }

    /// Cleans up a closed or errored I/O entry.
    fn cleanup(&mut self, token: usize, fd: RawFd) {
        self.poller.deregister(fd);
        self.io.remove(token).wake_all();
        sys_close(fd);
    }
}

/// Reads data from a file descriptor into a buffer.
///
/// Returns `true` if the file descriptor should be closed.
fn handle_read(fd: RawFd, buffer: &mut Vec<u8>) -> bool {
    let mut temp = [0u8; 1024];

    loop {
        let n = sys_read(fd, &mut temp);

        match n {
            (1..) => {
                buffer.extend_from_slice(&temp[..n as usize]);
            }
            0 => {
                return true;
            }
            _ => {
                let error = io::Error::last_os_error();

                if error.kind() == io::ErrorKind::WouldBlock {
                    break;
                } else {
                    return true;
                }
            }
        }
    }

    false
}

/// Writes buffered data to a file descriptor.
///
/// Returns `true` if the file descriptor should be closed.
fn handle_write(fd: RawFd, buffer: &mut Vec<u8>) -> bool {
    while !buffer.is_empty() {
        let n = sys_write(fd, buffer);

        if n > 0 {
            buffer.drain(..n as usize);
        } else if n < 0 {
            let err = io::Error::last_os_error();

            if err.kind() == io::ErrorKind::WouldBlock {
                break;
            } else {
                return true;
            }
        }
    }

    false
}
