# Windows Implementation Plan

## Overview

Implement `IShellItemImageFactory`-based thumbnail extraction in `src/platform/windows.rs`.
Uses the same Shell API that Explorer.exe calls — returns real thumbnails when available,
file-type icons as fallback.

## API Flow

```
CoInitializeEx(COINIT_APARTMENTTHREADED)
  └─ SHCreateItemFromParsingName(path, IShellItemImageFactory)
       └─ GetImage(SIZE { cx, cy }, SIIGBF_RESIZETOFIT) → HBITMAP
            └─ Extract pixels via GDI → BGRA → swap to RGBA
  └─ CoUninitialize()
```

## Steps

### 1. `to_wide_string(s: &str) -> Vec<u16>`

Convert Rust `&str` to null-terminated UTF-16 for Win32 `PCWSTR` args.

```rust
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
```

### 2. `generate_thumbnail(path, size) -> Result<Thumbnail, ThumbsError>`

Main entry point. Steps:

1. Validate file exists → `ThumbsError::FileNotFound`
2. `CoInitializeEx(None, COINIT_APARTMENTTHREADED)` — if returns `S_FALSE`, already initialized (ok)
3. `to_wide_string(path)`
4. `SHCreateItemFromParsingName(PCWSTR, None) → IShellItemImageFactory`
5. `GetImage(SIZE { cx: size.width, cy: size.height }, SIIGBF_RESIZETOFIT=0x0) → HBITMAP`
6. `hbitmap_to_rgba(hbitmap)` → `(rgba, w, h)`
7. Cleanup: `DeleteObject(hbitmap)`, `CoUninitialize()`
8. Return `Thumbnail::new(rgba, w, h)`

#### Error mapping

| Condition | Error |
|-----------|-------|
| `CoInitializeEx` fails | `PlatformError` |
| `SHCreateItemFromParsingName` fails (path not found) | `FileNotFound` |
| `SHCreateItemFromParsingName` fails (access denied, etc.) | `PlatformError` |
| `GetImage` fails | `ThumbnailGenerationFailed` |

### 3. `hbitmap_to_rgba(hbitmap) -> Result<(Vec<u8>, u32, u32), ThumbsError>`

Extract raw pixels from HBITMAP using GDI:

1. `GetObjectW(hbitmap)` → `BITMAP { bmWidth, bmHeight, bmBitsPixel, bmBits }`
2. `CreateCompatibleDC(None)` → memory DC
3. `SelectObject(dc, hbitmap)` → select bitmap into DC
4. Set up `BITMAPINFO` header:
   - `biSize = size_of::<BITMAPINFOHEADER>()`
   - `biWidth = bmWidth`
   - `biHeight = -bmHeight` (negative = top-down DIB)
   - `biPlanes = 1`
   - `biBitCount = 32`
   - `biCompression = BI_RGB`
5. `GetDIBits(dc, hbitmap, 0, height, buffer, DIB_RGB_COLORS)` → raw BGRA pixels
6. `SelectObject(dc, old_bitmap)`, `DeleteDC(dc)`
7. Convert BGRA → RGBA: swap bytes `[0] ↔ [2]` per pixel
8. Handle `bmHeight` negative (top-down) vs positive (bottom-up, flip rows)

### 4. Cargo.toml (already configured)

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_System_Com",
    "Win32_UI_Shell",
    "Win32_Graphics_Gdi",
] }
```

## Key Windows types needed

| Rust type | Windows type |
|-----------|--------------|
| `windows::Win32::System::Com::CoInitializeEx` | COM init |
| `windows::Win32::UI::Shell::SHCreateItemFromParsingName` | Shell item |
| `windows::Win32::UI::Shell::IShellItemImageFactory` | Thumbnail factory |
| `windows::Win32::Foundation::SIZE` | Dimensions |
| `windows::Win32::Graphics::Gdi::HBITMAP` | Bitmap handle |
| `windows::Win32::Graphics::Gdi::GetObjectW` | Bitmap info |
| `windows::Win32::Graphics::Gdi::CreateCompatibleDC` | Memory DC |
| `windows::Win32::Graphics::Gdi::SelectObject` | Select into DC |
| `windows::Win32::Graphics::Gdi::GetDIBits` | Read pixels |
| `windows::Win32::Graphics::Gdi::DeleteObject` | Cleanup |
| `windows::Win32::Graphics::Gdi::DeleteDC` | Cleanup |
| `windows::Win32::Graphics::Gdi::BITMAPINFO` | DIB header |
| `windows::Win32::Graphics::Gdi::BITMAPINFOHEADER` | DIB header |

## Testing

- GitHub Actions Windows runner: `cargo test` + CLI example
- Test with `notepad.exe` (has embedded icon) and a directory path
- Upload `_thumb.png` artifacts for visual inspection

## Flag: `SIIGBF_RESIZETOFIT` vs `SIIGBF_THUMBNAILONLY`

Default (0x0 = `SIIGBF_RESIZETOFIT`): matches Explorer behavior — real thumbnails when
a provider exists, file-type icon as fallback. This is the right default.

`SIIGBF_THUMBNAILONLY` would fail for .txt, .log, .md files with no thumbnail provider.
