# Cadentis

[![License](https://img.shields.io/badge/license-SSPL-blue.svg)](LICENSE)
![Dev Rust](https://img.shields.io/badge/Developed%20with-Rust%201.92.0-orange)
[![CI](https://github.com/Nebula-ecosystem/Cadentis/actions/workflows/ci.yml/badge.svg)](https://github.com/Nebula-ecosystem/Cadentis/actions/workflows/ci.yml)

**Cadentis** is the dedicated lightweight task orchestration runtime for the ***Nebula*** ecosystem, providing only the essential primitives required by the platform.

---

## 📊 Project Status

### ✅ Working Version

- [x] **Stable and usable version**
  - Fully functional async runtime
  - Reliable multi-threaded executor
  - Working network and filesystem I/O
  - Timers, sleep, timeout
  - Ergonomic macros (`main`, `test`, `join!`, `select!`)
  - macOS / Linux / Windows support

> 👉 This milestone is **done**. Cadentis works.

### 🚀 Next Priorities

- [ ] **Robustness**
  - [ ] Graceful shutdown
  - [ ] Task cancellation
  - [ ] Prevent task leaks

- [ ] **Performance**
  - [ ] Basic benchmarks
  - [ ] Scheduler fairness
  - [ ] Reduce allocations

- [ ] **Debug & Introspection**
  - [ ] Minimal runtime tracing
  - [ ] Task lifecycle visibility

- [ ] **API & Ergonomics**
  - [ ] Refine spawn / join / select
  - [ ] A few essential utility futures

---

## 🚀 Getting Started

This crate is not published on crates.io. Add it directly from GitHub:

```toml
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

## 📚 Examples

Cadentis includes **several practical examples** demonstrating how to use the runtime primitives in real-world scenarios:

- TCP servers and clients  
- Task spawning and coordination  
- Timers, sleep, and timeouts  
- Basic async I/O patterns  

Examples are available in the repository under the `examples/` directory.

Run an example with:

```
cargo run --example tcp_server
```

These examples are intentionally **minimal and focused**, showcasing how Cadentis primitives compose without hiding complexity behind heavy abstractions.

---

## 📖 Documentation

You can generate the full API documentation locally using Cargo:

```
cargo doc --open
```

This will build and open the documentation for Cadentis and all its public APIs in your browser.

> ⚠️ Cadentis is designed as a **low-level runtime**, so the documentation focuses on **clear contracts and primitives**, not high-level frameworks.

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

---

## 🤝 Contact

For questions, discussions, or contributions, feel free to reach out:

- **Discord**: enzoblain  
- **Email**: [enzoblain@proton.me](mailto:enzoblain@proton.me)