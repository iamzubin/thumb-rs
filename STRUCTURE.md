# Code Structure

## Project Layout

```
src/
  lib.rs              Public API: Thumbnail, ThumbnailScale, get_thumbnail()
  error.rs            ThumbsError enum (thiserror)
  platform/
    mod.rs            Conditional re-exports via #[cfg(target_os)]
    macos.rs          QLThumbnailGenerator implementation
    windows.rs        IShellItemImageFactory implementation
examples/
  cli.rs              CLI tool that generates thumbnail PNGs
tests/
  integration.rs      End-to-end tests using the image crate
```

## Public API

The library exposes three types and one function:

- **`get_thumbnail(path, scale)`** — main entry point. Returns `Result<Thumbnail, ThumbsError>`.
- **`Thumbnail`** — `{ rgba: Vec<u8>, width: u32, height: u32 }`. Raw RGBA8 pixels.
- **`ThumbnailScale(u32)`** — size multiplier on a 256px base. `1` = 256×256, `2` = 512×512, etc.
- **`ThumbsError`** — error enum covering file not found, platform errors, and generation failures.

## Platform Backends

Each platform module exports `generate_thumbnail(path, scale) -> Result<Thumbnail, ThumbsError>`.

### macOS (`src/platform/macos.rs`)

Uses Apple's `QLThumbnailGenerator` via the `objc2-quick-look-thumbnailing` crate.

1. Create `NSURL` from path
2. Build `QLThumbnailGenerationRequest` with target size
3. Call `generateBestRepresentationForRequest_completionHandler` (async callback)
4. Pump `CFRunLoop` until callback fires or 30s timeout
5. Extract `CGImage` → RGBA8 via `CGBitmapContext` (handles any input pixel format)
6. Un-premultiply alpha channel

### Windows (`src/platform/windows.rs`)

Uses the Shell API via the `windows` crate.

1. `CoInitializeEx(COINIT_APARTMENTTHREADED)` — required for COM
2. `SHCreateItemFromParsingName(path)` → `IShellItemImageFactory`
3. `GetImage(size, SIIGBF(0))` → `HBITMAP`
4. Extract pixels: `GetObjectW` → `GetDIBits` → raw BGRA buffer
5. Swap B↔R channels → RGBA
6. Flip rows (bottom-up → top-down)
7. Cleanup: `DeleteObject`, `DeleteDC`, `CoUninitialize`

Uses `SIIGBF(0)` (default flags = `SIIGBF_RESIZETOFIT`) — same behavior as Explorer:
real thumbnails when available, file-type icons as fallback.

## Dependencies

| Dep | Purpose |
|-----|---------|
| `thiserror` | Error types |
| `objc2-*` (macos) | ObjC FFI for QuickLook, CoreGraphics |
| `dispatch2` (macos) | GCD primitives |
| `windows` (win32) | COM, Shell, GDI APIs |
| `image` (dev) | PNG/JPEG encoding in tests/examples |
| `tempfile` (dev) | Test fixtures |
