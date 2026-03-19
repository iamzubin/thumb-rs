# thumb-rs

High-performance cross-platform thumbnail extraction for macOS and Windows. Returns raw RGBA pixels — works with **any file type** the OS can preview.

## Features

- **Any file type** — not just images. Uses OS-native APIs:
  - macOS: `QLThumbnailGenerator` (same as Finder QuickLook)
  - Windows: `IShellItemImageFactory` (same as Explorer)
- **Raw pixels** — no format lock-in. Returns `Vec<u8>` RGBA data.
- **Explorer/Finder quality** — video frames via VLC, PDF previews, document thumbnails.
- **No `image` crate dependency** — library is a thin wrapper around OS APIs. You encode to PNG/JPEG yourself.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thumb-rs = "0.1"
```

No platform-specific features to enable — the correct backend is selected automatically via `#[cfg(target_os)]`. macOS and Windows are both supported; Linux is not.

## Usage

```rust
use thumb_rs::{get_thumbnail, ThumbnailScale};

// Generate thumbnail for any file type (scale 1 = 256×256, scale 2 = 512×512, etc.)
let thumb = get_thumbnail("video.mp4", ThumbnailScale(2))?;

// thumb.rgba  — raw RGBA8 pixel data
// thumb.width  — actual width
// thumb.height — actual height

// Encode to PNG (using the `image` crate as a dev-dependency)
use image::{ImageBuffer, Rgba};
let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
    thumb.width, thumb.height, thumb.rgba
).unwrap();
img.save("thumb.png")?;
```

### CLI example

```sh
cargo run --example cli -- /path/to/file.pdf --scale 2
# creates /path/to/file_thumb.png
```

### Thumbnail scale

```rust
use thumb_rs::ThumbnailScale;

ThumbnailScale::default();  // 256×256 (scale 1)
ThumbnailScale(2);          // 512×512
ThumbnailScale(4);          // 1024×1024
```

## How it works

| Platform | API | Returns |
|----------|-----|---------|
| macOS | `QLThumbnailGenerator` | `CGImage` → RGBA8 |
| Windows | `IShellItemImageFactory::GetImage` | `HBITMAP` → RGBA8 |

On macOS, `QLThumbnailGenerator` is called synchronously via `dispatch2::Semaphore` (wraps the async callback). The CGImage is converted to RGBA8 via `CGBitmapContext`, handling any input pixel format (8-bit, 16-bit half-float, etc.).

On Windows, `IShellItemImageFactory::GetImage` returns an `HBITMAP`. Pixels are extracted via `GetDIBits` and converted from BGRA to RGBA. Uses default flags (`SIIGBF_RESIZETOFIT`), matching Explorer behavior — real thumbnails when available, icons as fallback.

## Development

```sh
cargo build
cargo test
cargo clippy -- -D warnings
```

CI runs on both macOS (ARM) and Windows via GitHub Actions. Windows CI uploads thumbnail PNGs as artifacts for visual inspection.

## License

MIT
