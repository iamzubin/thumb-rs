## AGENTS.md

This project is a high-performance, cross-platform Rust library for extracting thumbnails from file paths. To assist AI agents (like GitHub Copilot, Cursor, or Windsurf) in navigating and contributing to `thumbs-rs`, follow these standards.

---

### 🟢 Tech Stack & Context
* **Language:** Rust (Stable)
* **Target:** Cross-platform (Linux, macOS, Windows)
* **Core Logic:** File I/O, image decoding/scaling, and OS-specific thumbnail API integration.

### 🦀 Rust-Specific Agent Guidelines
* **Strict Typing:** Always prefer explicit types in function signatures to help agents infer trait bounds correctly.
* **Error Handling:** Use `thiserror` for library-level errors. Ensure all public functions return a `Result<T, ThumbsError>`.
* **Concurrency:** Use `Send + Sync` bounds where necessary, as thumbnail generation is often parallelized.
* **Performance:** Favor `std::path::Path` or `AsRef<Path>` for input arguments to avoid unnecessary string allocations.

### 📂 Repository Structure
* `/src/lib.rs`: Main entry point and public API.
* `/src/platform/`: OS-specific implementations (e.g., `macos.rs`, `windows.rs`).
* `/examples/`: Reference implementations for quick context.

### 🛠 Development Commands
* **Build:** `cargo build`
* **Test:** `cargo test`
* **Check Types:** `cargo check`
* **Lint:** `cargo clippy -- -D warnings`

---

Would you like me to generate a `contributing.md` or a basic `src/lib.rs` boilerplate to get the project started?
