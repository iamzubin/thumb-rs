use std::path::PathBuf;
use tempfile::TempDir;
use thumb_rs::{get_thumbnail, ThumbnailScale};

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

    let thumb = get_thumbnail(&test_image, ThumbnailScale::default())
        .expect("Should generate thumbnail from PNG");

    assert!(!thumb.rgba.is_empty(), "RGBA data should not be empty");
    assert_eq!(thumb.rgba.len(), (thumb.width * thumb.height * 4) as usize);
    assert!(thumb.width <= 256, "Width should be within bounds");
    assert!(thumb.height <= 256, "Height should be within bounds");
}

#[test]
fn test_thumbnail_respects_scale() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "large.png", 1000, 800);

    let scale = ThumbnailScale(1);
    let thumb = get_thumbnail(&test_image, scale).expect("Should generate thumbnail");

    let px = scale.px();
    assert!(
        thumb.width <= px && thumb.height <= px,
        "Image should fit within bounds (got {}x{}, max {}x{})",
        thumb.width,
        thumb.height,
        px,
        px
    );
}

#[test]
fn test_thumbnail_file_not_found() {
    let result = get_thumbnail("/nonexistent/path/image.png", ThumbnailScale::default());
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
        get_thumbnail(&test_image, ThumbnailScale::default()).expect("Should generate thumbnail");

    // Verify RGBA structure
    let pixel_count = (thumb.width * thumb.height) as usize;
    assert_eq!(thumb.rgba.len(), pixel_count * 4);

    // Each pixel should have 4 bytes (R, G, B, A)
    for chunk in thumb.rgba.chunks(4) {
        assert_eq!(chunk.len(), 4, "Each pixel must have exactly 4 bytes");
    }
}

#[test]
fn test_thumbnail_with_various_scales() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "scale_test.png", 512, 512);

    let scales = [ThumbnailScale(1), ThumbnailScale(2)];

    for scale in scales {
        let thumb = get_thumbnail(&test_image, scale)
            .unwrap_or_else(|e| panic!("Should generate thumbnail at scale {}: {}", scale.0, e));
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

    let thumb = get_thumbnail(&path, ThumbnailScale::default())
        .expect("Should generate thumbnail from JPEG");

    assert!(!thumb.rgba.is_empty());
}
