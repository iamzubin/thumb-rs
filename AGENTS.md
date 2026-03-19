## AGENTS.md

This project is a high-performance, cross-platform Rust library for extracting thumbnails from **any file type** the OS can preview. Returns raw RGBA pixel data — users encode to PNG/JPEG themselves.

---

### Tech Stack & Context
* **Language:** Rust (Stable)
* **Target:** macOS, Windows (no Linux)
* **Core Logic:** OS-native thumbnail APIs, NOT the `image` crate for decoding.
* **macOS API:** `QLThumbnailGenerator` via `objc2-quick-look-thumbnailing`
* **Windows API:** `IShellItemImageFactory` via `windows` crate
* **Return type:** `Thumbnail { rgba: Vec<u8>, width: u32, height: u32 }` — raw RGBA8

### Rust-Specific Agent Guidelines
* **Strict Typing:** Always prefer explicit types in function signatures.
* **Error Handling:** Use `thiserror` for library-level errors. All public functions return `Result<T, ThumbsError>`.
* **Concurrency:** Use `Send + Sync` bounds where necessary.
* **Performance:** Favor `std::path::Path` or `AsRef<Path>` for input arguments.
* **No `image` crate in core deps:** Library returns raw pixels. `image` is dev-dependency only (tests/examples).
* **Platform deps are conditional:** macOS gets objc2 ecosystem, Windows gets `windows` crate. Neither is compiled on the other platform.

### Repository Structure
* `/src/lib.rs`: Public API — `Thumbnail`, `ThumbnailSize`, `get_thumbnail()`.
* `/src/error.rs`: `ThumbsError` enum with `thiserror`.
* `/src/platform/mod.rs`: Conditional module re-exports (`#[cfg(target_os)]`).
* `/src/platform/macos.rs`: `QLThumbnailGenerator` implementation (stub — has TODO comments).
* `/src/platform/windows.rs`: `IShellItemImageFactory` implementation (stub — has TODO comments).
* `/examples/cli.rs`: CLI tool that generates thumbnail PNGs.
* `/tests/integration.rs`: Integration tests using `image` crate (dev dep) to create test images.

### Key Design Decisions
1. **Any file type:** Not limited to images. Uses OS-native APIs that support PDFs, videos, docs, etc.
2. **Raw RGBA output:** No format lock-in. Library is a thin wrapper around OS APIs.
3. **Windows default flags:** Uses `SIIGBF_RESIZETOFIT` (0x0), NOT `SIIGBF_THUMBNAILONLY`. This matches Explorer behavior — invokes VLC/Office thumbnail providers, falls back to file icons.
4. **macOS sync wrapper:** Uses `dispatch2::Semaphore` to block on async `QLThumbnailGenerator` callback.

### Development Commands
* **Build:** `cargo build`
* **Test:** `cargo test`
* **Check:** `cargo check`
* **Lint:** `cargo clippy -- -D warnings`

---
