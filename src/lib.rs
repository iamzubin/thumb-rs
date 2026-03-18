use std::path::Path;

pub mod error;
pub use error::ThumbsError;

#[cfg(target_os = "macos")]
pub mod platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbnailSize {
    pub width: u32,
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
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

pub fn get_thumbnail<P: AsRef<Path>>(
    file_path: P,
    output_path: P,
    size: ThumbnailSize,
) -> Result<(), ThumbsError> {
    let file_path = file_path.as_ref();
    let output_path = output_path.as_ref();

    #[cfg(target_os = "macos")]
    {
        platform::macos::generate_thumbnail(file_path, output_path, size)
    }

    #[cfg(windows)]
    {
        platform::windows::generate_thumbnail(file_path, output_path, size)
    }

    #[cfg(target_os = "linux")]
    {
        platform::linux::generate_thumbnail(file_path, output_path, size)
    }
}

pub fn get_thumbnail_bytes<P: AsRef<Path>>(
    file_path: P,
    size: ThumbnailSize,
) -> Result<Vec<u8>, ThumbsError> {
    let file_path = file_path.as_ref();

    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_FORMATS.contains(&extension.as_str()) {
        return Err(ThumbsError::UnsupportedFormat(extension));
    }

    let img = image::open(file_path).map_err(|e| ThumbsError::ImageError(e.to_string()))?;

    let thumbnail = img.thumbnail(size.width, size.height);

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);

    thumbnail
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| ThumbsError::ImageError(e.to_string()))?;

    Ok(buffer)
}

pub const SUPPORTED_FORMATS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "ico", "svg",
];

pub fn is_supported<P: AsRef<Path>>(file_path: P) -> bool {
    let extension = file_path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    SUPPORTED_FORMATS.contains(&extension.as_str())
}

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
    fn test_supported_formats() {
        assert!(is_supported("test.png"));
        assert!(is_supported("test.jpg"));
        assert!(is_supported("test.JPEG"));
        assert!(!is_supported("test.txt"));
        assert!(!is_supported("test"));
    }
}
