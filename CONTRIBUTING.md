# Contributing to Async

Thank you for considering contributing to **Async**!  
We welcome all kinds of contributions: bug reports, feature requests, documentation improvements, examples, or code enhancements.

---

## 🛠 Getting Started

1. **Fork the repository**  
2. **Clone your fork** locally:  
```
git clone https://github.com/enzoblain/Async.git
cd Async
```  
3. **Create a new branch** for your feature or fix:  
```
git checkout -b my-feature
```

---

## 📝 Code Guidelines

- **Rust best practices**: Follow idiomatic Rust patterns.
- **Safety first**: Avoid `unsafe` unless strictly necessary and documented.
- **Async & parallel code**: Use `async/await` for network operations, and `spawn_blocking` or thread pools for CPU-bound work.
- **Tests**: Every new feature or bug fix should include tests.
- **Error handling**: Do not use `unwrap()` or `expect()` in public APIs. Prefer `Result<T, E>` or `Option<T>` and propagate errors properly.
- **Documentation**: Public APIs should have doc comments with examples (`///`), and update examples if relevant.

---

## ✅ Testing

Before submitting a pull request, ensure that all tests pass:

```
cargo test
```

Run formatting checks:

```
cargo fmt -- --check
```

Run clippy linter for code style warnings:

```
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 🖇 Pull Requests

1. Push your branch to your fork:  
```
git push origin my-feature
```  
2. Open a **Pull Request** to the `main` branch of the main repository.  
3. Include a **clear description** of what your PR does, referencing any related issues.  
4. PRs will be reviewed by the maintainers; we may request changes before merging.

---

## 💡 Reporting Issues

If you find a bug or have a feature request, please open an **issue**:

- Provide a **clear description** of the problem or request.  
- Include **steps to reproduce**, **expected behavior**, and **actual behavior** for bugs.  
- Include **system information** (OS, Rust version, crate version) if relevant.

---

## 🎉 Thank You!

We appreciate your contributions!  
Every bug report, PR, and suggestion helps make **Async** better and more robust.
