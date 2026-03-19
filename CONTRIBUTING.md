# Contributing

## Development Setup

Requires Rust stable. Platform-specific code only compiles on its target OS.

```sh
git clone https://github.com/iamzubin/thumb-rs.git
cd thumb-rs
cargo build
cargo test
```

## Running Checks

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All three must pass before submitting a PR.

## Project Structure

See [STRUCTURE.md](STRUCTURE.md) for an overview of the codebase.

## Platform Code

- **macOS**: `src/platform/macos.rs` — uses `QLThumbnailGenerator` via `objc2` bindings.
- **Windows**: `src/platform/windows.rs` — uses `IShellItemImageFactory` via the `windows` crate.

Platform code is selected at compile time via `#[cfg(target_os)]`. Each module exports
`generate_thumbnail(path, scale) -> Result<Thumbnail, ThumbsError>`.

## Testing on Windows

Windows CI runs on every push via GitHub Actions. There's also a manual workflow
(**Actions → Windows Thumbnail Test**) that downloads a test `.exe` and extracts
its icon thumbnail at various scales. Artifacts are uploaded as PNGs for visual review.

## Adding a New Platform Backend

1. Create `src/platform/<platform>.rs`
2. Add `#[cfg(target_os = "<platform>")] pub mod <platform>;` to `src/platform/mod.rs`
3. Add a `#[cfg(target_os = "<platform>")]` block in `src/lib.rs::get_thumbnail()`
4. Export `generate_thumbnail(path, scale) -> Result<Thumbnail, ThumbsError>`

## License

By contributing, you agree your code will be licensed under the MIT License.
