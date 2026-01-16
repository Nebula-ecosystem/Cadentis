# Cadentis

[![License](https://img.shields.io/badge/license-SSPL-blue.svg)](LICENSE)
![Dev Rust](https://img.shields.io/badge/Developed%20with-Rust%201.92.0-orange)
[![CI](https://github.com/Nebula-ecosystem/Cadentis/actions/workflows/ci.yml/badge.svg)](https://github.com/Nebula-ecosystem/Cadentis/actions/workflows/ci.yml)

**Cadentis** is the dedicated lightweight task orchestration runtime for the ***Nebula*** ecosystem, providing only the essential primitives required by the platform.

---

## 📊 Project Status

- [x] **Runtime & Scheduling**
  - [x] Task Spawning (async, background)
  - [x] Event Loop (block_on, scheduling)
  - [x] Thread-Local Context

- [x] **I/O & Filesystem**
  - [x] Async File (non-blocking read/write)
  - [x] Async Folder (mkdir, recursive creation)
  - [x] TCP Listener (accept connections)
  - [x] TCP Stream (read/write, echo)

- [x] **Reactor & Events**
  - [x] Kqueue Integration (macOS)
  - [x] Timer Events (sleep, timeout)
  - [x] Event Registration (read/write/timer)

- [x] **Time & Utilities**
  - [x] Sleep Future (async delay)
  - [x] Timeout Combinator (deadline for tasks)
  - [x] Time Measurement (benchmark async ops)
  - [x] Retry Utility (repeated attempts)

- [x] **Multithreading**
  - [x] Multi-threaded Executor (work-stealing, thread pool)
  - [x] Thread-safe Context (Arc, Mutex)
  - [x] Cross-thread Task Spawning
  - [x] Synchronization Primitives (Mutex, Condvar, etc.)
  - [x] Structured Concurrency Primitives (join, select)

- [x] **Macros & Ergonomics**
  - [x] `cadentis::main` proc-macro
  - [x] `cadentis::test` proc-macro
  - [x] `join!` / `select!` macros (extensible / variadic for arbitrary number of futures)

- [ ] **Extensibility**
  - [ ] Windows Support (IOCP)
  - [x] Linux Support (epoll)

---

## 🚀 Getting Started

This crate is not published on crates.io. Add it directly from GitHub:

``` toml
[dependencies]
cadentis = { git = "https://github.com/Nebula-ecosystem/Cadentis", package = "cadentis" }
```

---

## 📡 Example: TcpListener

Accept and handle an incoming TCP connection using Cadentis async I/O and task scheduling:

```rust
use cadentis::net::tcp::TcpListener;
use cadentis::task::spawn;

#[cadentis::main]
async fn main() {
    // Bind a TCP listener
    let listener = TcpListener::bind("127.0.0.1:8080")
        .expect("Failed to bind listener");

    // Spawn a task to handle a single connection
    spawn(async move {
        let (stream, _) = listener
            .accept()
            .await
            .expect("Failed to accept incoming connection");

        let mut buf = [0u8; 4];
        let n = stream
            .read(&mut buf)
            .await
            .expect("Failed to read from stream");

        if &buf[..n] == b"ping" {
            stream
                .write_all(b"pong")
                .await
                .expect("Failed to write response");
        }
    })
    .await;
}
```

---


## 🦀 Rust Version

- **Developed with**: Rust 1.92.0
- **MSRV**: Rust 1.92.0 (may increase in the future)

---

## 📄 License Philosophy

Cadentis is licensed under the **Server Side Public License (SSPL) v1**.

This license is intentionally chosen to protect the integrity of the Nebula ecosystem.  
While the project is fully open for **contribution, improvement, and transparency**,  
SSPL prevents third parties from creating competing platforms, proprietary versions,  
or commercial services derived from the project.

Nebula is designed to grow as **one unified, community-driven network**.  
By using SSPL, we ensure that:

- all improvements remain open and benefit the ecosystem,  
- the network does not fragment into multiple incompatible forks,  
- companies cannot exploit the project without contributing back,  
- contributors retain full access to the entire codebase.


In short, SSPL ensures that Cadentis — and the Nebula ecosystem built on top of it —  
remains **open to the community, but protected from fragmentation and exploitation**.

## 🤝 Contact

For questions, discussions, or contributions, feel free to reach out:

- **Discord**: enzoblain
- **Email**: [enzoblain@proton.me](mailto:enzoblain@proton.me)