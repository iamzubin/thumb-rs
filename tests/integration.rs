use std::path::PathBuf;
use tempfile::TempDir;
use thumb_rs::{get_thumbnail, ThumbnailSize};

/// Create a test PNG image using the `image` crate (dev dependency).
fn create_test_image(dir: &TempDir, name: &str, width: u32, height: u32) -> PathBuf {
    let path = dir.path().join(name);
    let img = image::RgbaImage::from_pixel(width, height, image::Rgba([100, 150, 200, 255]));
    img.save(&path).expect("Should save test image");
    path
}

#[test]
fn test_thumbnail_from_png() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "test.png", 800, 600);

    let thumb = get_thumbnail(&test_image, ThumbnailSize::square(128))
        .expect("Should generate thumbnail from PNG");

    assert!(!thumb.rgba.is_empty(), "RGBA data should not be empty");
    assert_eq!(thumb.rgba.len(), (thumb.width * thumb.height * 4) as usize);
    assert!(thumb.width <= 128, "Width should be within bounds");
    assert!(thumb.height <= 128, "Height should be within bounds");
}

#[test]
fn test_thumbnail_respects_size() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "large.png", 1000, 800);

    let size = ThumbnailSize::new(100, 200);
    let thumb = get_thumbnail(&test_image, size).expect("Should generate thumbnail");

    assert!(
        thumb.width <= size.width || thumb.height <= size.height,
        "Image should fit within bounds (got {}x{}, max {}x{})",
        thumb.width,
        thumb.height,
        size.width,
        size.height
    );
}

#[test]
fn test_thumbnail_file_not_found() {
    let result = get_thumbnail("/nonexistent/path/image.png", ThumbnailSize::default());
    assert!(result.is_err());
    match result.unwrap_err() {
        thumb_rs::ThumbsError::FileNotFound(_) => {}
        other => panic!("Expected FileNotFound, got: {:?}", other),
    }
}

#[test]
fn test_thumbnail_rgba_is_valid() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "pixel_test.png", 64, 64);

    let thumb =
        get_thumbnail(&test_image, ThumbnailSize::square(32)).expect("Should generate thumbnail");

    // Verify RGBA structure
    let pixel_count = (thumb.width * thumb.height) as usize;
    assert_eq!(thumb.rgba.len(), pixel_count * 4);

    // Each pixel should have 4 bytes (R, G, B, A)
    for chunk in thumb.rgba.chunks(4) {
        assert_eq!(chunk.len(), 4, "Each pixel must have exactly 4 bytes");
    }
}

#[test]
fn test_thumbnail_with_various_sizes() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "size_test.png", 512, 512);

    let sizes = [
        ThumbnailSize::square(64),
        ThumbnailSize::square(128),
        ThumbnailSize::square(256),
        ThumbnailSize::new(100, 50),
    ];

    for size in sizes {
        let thumb = get_thumbnail(&test_image, size)
            .unwrap_or_else(|e| panic!("Should generate thumbnail at {:?}: {}", size, e));
        assert!(!thumb.rgba.is_empty());
        assert!(thumb.width > 0 && thumb.height > 0);
    }
}

#[test]
fn test_thumbnail_jpeg_input() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.jpg");
    let img = image::RgbImage::from_pixel(400, 300, image::Rgb([255, 0, 0]));
    img.save(&path).expect("Should save JPEG test image");

    let thumb = get_thumbnail(&path, ThumbnailSize::square(100))
        .expect("Should generate thumbnail from JPEG");

    assert!(!thumb.rgba.is_empty());
}
