use std::path::Path;

pub mod error;
pub use error::ThumbsError;

#[cfg(target_os = "macos")]
pub mod platform;

#[cfg(target_os = "windows")]
pub mod platform;

/// Raw thumbnail data returned by the library.
///
/// Contains uncompressed RGBA8 pixel data in row-major order.
/// Each pixel is 4 bytes: [R, G, B, A].
/// Total byte length = `width * height * 4`.
///
/// To encode to PNG/JPEG, use the `image` crate or your preferred encoder:
/// ```ignore
/// use image::{ImageBuffer, Rgba};
/// let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
///     thumbnail.width, thumbnail.height, thumbnail.rgba
/// ).unwrap();
/// img.save("output.png").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Thumbnail {
    /// Raw RGBA8 pixel data, row-major, no padding.
    /// Length = width * height * 4.
    pub rgba: Vec<u8>,

    /// Width of the thumbnail in pixels.
    pub width: u32,

    /// Height of the thumbnail in pixels.
    pub height: u32,
}

impl Thumbnail {
    /// Create a new Thumbnail from raw pixel data.
    ///
    /// # Panics
    /// Panics if `rgba.len() != (width * height * 4) as usize`.
    pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(
            rgba.len(),
            (width as usize) * (height as usize) * 4,
            "rgba length must be width * height * 4"
        );
        Self {
            rgba,
            width,
            height,
        }
    }
}

/// Desired thumbnail dimensions in pixels.
///
/// The OS will return an image that fits within these bounds while
/// preserving the original aspect ratio. The actual output may be
/// smaller than the requested size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbnailSize {
    /// Maximum width in pixels.
    pub width: u32,

    /// Maximum height in pixels.
    pub height: u32,
}

impl Default for ThumbnailSize {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
        }
    }
}

impl ThumbnailSize {
    /// Create a custom thumbnail size.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Create a square thumbnail size.
    pub fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

/// Generate a thumbnail for any file type the OS can preview.
///
/// This is the main entry point. On macOS, it uses QuickLook (`QLThumbnailGenerator`).
/// On Windows, it uses the Shell API (`IShellItemImageFactory`).
///
/// Both implementations produce the SAME thumbnails you see in Finder/Explorer —
/// including video frames (via VLC), PDF previews, document thumbnails, etc.
///
/// # Arguments
/// * `file_path` — Path to the source file. Can be any type (image, video, PDF, etc.).
/// * `size` — Maximum thumbnail dimensions. Defaults to 256×256.
///
/// # Returns
/// A `Thumbnail` containing raw RGBA8 pixel data. Encode to PNG/JPEG yourself
/// using the `image` crate or any encoder.
///
/// # Errors
/// - `ThumbsError::FileNotFound` if the file doesn't exist.
/// - `ThumbsError::ThumbnailGenerationFailed` if the OS can't generate a thumbnail.
/// - `ThumbsError::PlatformError` for OS-level failures (COM errors, etc.).
/// - `ThumbsError::PlatformNotSupported` on unsupported platforms (e.g., Linux).
///
/// # Example
/// ```ignore
/// use thumb_rs::{get_thumbnail, ThumbnailSize};
///
/// let thumb = get_thumbnail("video.mp4", ThumbnailSize::square(256))?;
/// println!("Got {}x{} thumbnail ({} bytes)", thumb.width, thumb.height, thumb.rgba.len());
/// ```
pub fn get_thumbnail<P: AsRef<Path>>(
    file_path: P,
    size: ThumbnailSize,
) -> Result<Thumbnail, ThumbsError> {
    let file_path = file_path.as_ref();

    if !file_path.exists() {
        return Err(ThumbsError::FileNotFound(file_path.display().to_string()));
    }

    #[cfg(target_os = "macos")]
    {
        platform::macos::generate_thumbnail(file_path, size)
    }

    #[cfg(target_os = "windows")]
    {
        platform::windows::generate_thumbnail(file_path, size)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(ThumbsError::PlatformNotSupported)
    }
}

// ---------------------------------------------------------------------------
// Optional encoding helpers (behind feature flags, to be added later)
// ---------------------------------------------------------------------------

// #[cfg(feature = "png")]
// impl Thumbnail {
//     /// Encode this thumbnail as PNG bytes.
//     pub fn to_png(&self) -> Result<Vec<u8>, ThumbsError> { ... }
// }
//
// #[cfg(feature = "jpeg")]
// impl Thumbnail {
//     /// Encode this thumbnail as JPEG bytes with the given quality (1-100).
//     pub fn to_jpeg(&self, quality: u8) -> Result<Vec<u8>, ThumbsError> { ... }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_size_default() {
        let size = ThumbnailSize::default();
        assert_eq!(size.width, 256);
        assert_eq!(size.height, 256);
    }

    #[test]
    fn test_thumbnail_size_square() {
        let size = ThumbnailSize::square(128);
        assert_eq!(size.width, 128);
        assert_eq!(size.height, 128);
    }

    #[test]
    fn test_thumbnail_new_valid() {
        let rgba = vec![0u8; 256 * 256 * 4];
        let thumb = Thumbnail::new(rgba, 256, 256);
        assert_eq!(thumb.width, 256);
        assert_eq!(thumb.height, 256);
    }

    #[test]
    #[should_panic(expected = "rgba length must be")]
    fn test_thumbnail_new_invalid_size() {
        let rgba = vec![0u8; 100]; // wrong size
        Thumbnail::new(rgba, 256, 256);
    }
}
