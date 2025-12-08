# Executor

[![Rust](https://img.shields.io/badge/Rust-1.91-orange?logo=rust)](https://www.rust-lang.org/)    
[![License](https://img.shields.io/badge/license-SSPL-blue.svg)](LICENSE)

---

**Executor** is a lightweight, predictable task runner written in **Rust**.

Designed for **systems programming** and **application-level coordination**,  
Executor provides the fundamental building blocks to schedule, drive, and monitor  
units of work — with or without async.

It aims to be a **clear, minimal foundation** for building custom schedulers,  
service runtimes, and deterministic task pipelines.

---

# 🧩 Purpose of Executor

Modern services — networking stacks, embedded runtimes, job systems, or orchestrators —
often need a **deterministic, observable execution loop** they control end-to-end.

Executor focuses on:

- predictable task progression (no hidden threads)  
- simple scheduling hooks you can extend  
- explicit ownership of work queues and timers  
- compatibility with both synchronous code and async primitives  

This makes it a **runtime substrate** for schedulers, workers, and pipelines where
latency budgets and ordering guarantees matter.

---

# ✨ Key Features

- 🧱 **Minimal Execution Core**  
  Straightforward loop + queue primitives; easy to read, reason about, and extend.

- ⚙️ **Deterministic Scheduling**  
  No hidden threads; ordering and progression are explicit and testable.

- 🧩 **Sync First, Async Ready**  
  Start with synchronous tasks; evolve toward async executors or reactors as needed.

- 🌐 **No-Std (Planned)**  
  A `no-std` mode is on the roadmap for constrained targets; feature flag is stubbed today.

- 🚀 **Performance-Conscious**  
  Favor O(1) enqueue/progress operations with room for instrumentation.

- 🔧 **Composable Hooks**  
  Add metrics, tracing, backpressure, or priority policies without wrestling a black box.

- 🧪 **Testing-Oriented**  
  Deterministic runs enable focused unit and integration tests for schedulers and jobs.

---

# 🧭 Project Status

🚧 **Active Development**

Executor is evolving toward a small, hackable runtime core.

Current focus areas include:

- single-threaded execution loop with pluggable queues  
- task lifecycle hooks (start/finish/error)  
- timers and delayed work primitives  
- optional async bridge and waking strategy  
- observability: metrics, traces, and backpressure signals  

Contributions and feedback are highly encouraged.

---

# 📦 Installation

During development:

``` toml
[dependencies]
executor = { path = "https://github.com/enzoblain/Executor" }
```

Once published:

``` toml
[dependencies]
executor = "0.1"
```

Planned `no-std` mode (feature flag stubbed, support coming later):

``` toml
[dependencies]
executor = { version = "0.1", features = ["no-std"] }
```

---

# 🤝 Contributing

Contributions are welcome — especially regarding:

- scheduling policies and queue strategies  
- async integration and waking  
- instrumentation (metrics, tracing)  
- backpressure and cancellation  
- documentation & examples  

Typical workflow:

``` sh
cargo fmt
cargo clippy
cargo test --workspace
```

See `CONTRIBUTING.md` for details.

---

# 📄 License Philosophy

Executor is licensed under the **Server Side Public License (SSPL) v1**.

This license ensures the runtime remains **open** while preventing  
proprietary forks or commercial services from exploiting the project  
without contributing back.

It protects Executor in contexts where determinism, transparency, and ecosystem integrity matter.

---

# 📬 Contact

**Discord:** enzoblain  
**Email:** enzoblain@proton.me  

Open to discussions, improvements, and architecture/design questions.