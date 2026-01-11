use super::command::Command;
use super::event::Event;
use super::io::IoEntry;
use super::poller::Poller;
use super::poller::platform::{sys_close, sys_read, sys_write};
use super::timer::TimerEntry;
use crate::reactor::io::Waiting;
use crate::utils::Slab;

use std::collections::BinaryHeap;
use std::io;
use std::os::fd::RawFd;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

pub(crate) struct Reactor {
    receiver: Receiver<Command>,

    poller: Poller,
    events: Vec<Event>,

    timers: BinaryHeap<TimerEntry>,
    io: Slab<IoEntry>,
}

impl Reactor {
    pub(crate) fn new() -> (Self, Sender<Command>) {
        let (transmitter, receiver) = channel();
        let poller = Poller::new();
        let events = Vec::with_capacity(64);
        let timers = BinaryHeap::new();
        let io = Slab::new(64);

        (
            Self {
                receiver,
                poller,
                events,
                timers,
                io,
            },
            transmitter,
        )
    }

    pub(crate) fn run(&mut self) -> io::Result<()> {
        loop {
            let events: Vec<Event> = self.events.drain(..).collect();
            for event in events {
                self.handle_event(event);
            }

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
                        self.poller.wake();
                    }
                    Command::Shutdown => {
                        return Ok(());
                    }
                    Command::Wake => {
                        self.poller.wake();
                    }
                }
            }

            let timeout = self
                .timers
                .peek()
                .map(|t| t.deadline.saturating_duration_since(Instant::now()));

            self.poller.poll(&mut self.events, timeout)?;

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

    fn handle_event(&mut self, event: Event) {
        let mut should_close = false;
        let mut fd = None;
        let mut new_interest = None;

        {
            let entry = self.io.get_mut(event.token);
            match entry {
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

                IoEntry::Stream(stream) => {
                    fd = Some(stream.fd);

                    if event.readable {
                        if handle_read(stream.fd, &mut stream.in_buffer) {
                            should_close = true;
                            fd = Some(stream.fd);
                        }
                    }

                    if !should_close && event.writable {
                        if handle_write(stream.fd, &mut stream.out_buffer) {
                            should_close = true;
                            fd = Some(stream.fd);
                        }
                    }

                    new_interest = Some(stream.interest());
                }
            }
        }

        if let Some(fd) = fd {
            if should_close {
                self.cleanup(event.token, fd);
            } else {
                if let Some(ni) = new_interest {
                    self.poller.reregister(fd, event.token, ni);
                }
            }
        }
    }

    fn cleanup(&mut self, token: usize, fd: RawFd) {
        self.poller.deregister(fd);
        self.io.remove(token).wake_all();

        sys_close(fd);
    }
}

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
