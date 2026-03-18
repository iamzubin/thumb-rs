use image::GenericImageView;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use thumb_rs::{get_thumbnail, get_thumbnail_bytes, is_supported, ThumbnailSize};

#[test]
fn test_supported_formats() {
    assert!(is_supported("test.png"));
    assert!(is_supported("test.jpg"));
    assert!(is_supported("test.jpeg"));
    assert!(is_supported("test.gif"));
    assert!(is_supported("test.bmp"));
    assert!(is_supported("test.webp"));
}

#[test]
fn test_unsupported_format() {
    assert!(!is_supported("test.txt"));
    assert!(!is_supported("test.pdf"));
    assert!(!is_supported("test.doc"));
}

#[test]
fn test_thumbnail_bytes_creates_valid_png() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "test.png");

    let bytes = get_thumbnail_bytes(&test_image, ThumbnailSize::square(128))
        .expect("Should generate thumbnail bytes");

    assert!(!bytes.is_empty(), "Thumbnail bytes should not be empty");

    assert_eq!(
        &bytes[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        "Should be a valid PNG file"
    );
}

#[test]
fn test_thumbnail_saves_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "input.png");
    let output_path = temp_dir.path().join("output.png");

    get_thumbnail(&test_image, &output_path, ThumbnailSize::square(64))
        .expect("Should generate thumbnail");

    assert!(
        output_path.exists(),
        "Output file should exist at {}",
        output_path.display()
    );

    let metadata = fs::metadata(&output_path).expect("Should read metadata");
    assert!(metadata.len() > 0, "Output file should not be empty");
}

#[test]
fn test_thumbnail_respects_size() {
    let temp_dir = TempDir::new().unwrap();
    let test_image = create_test_image(&temp_dir, "large.png");
    let output_path = temp_dir.path().join("resized.png");

    let size = ThumbnailSize::new(100, 200);
    get_thumbnail(&test_image, &output_path, size).expect("Should generate thumbnail");

    let img = image::open(&output_path).expect("Should open output image");
    let (width, height) = img.dimensions();

    assert!(
        width <= size.width || height <= size.height,
        "Image should be resized (got {}x{}, expected max {}x{})",
        width,
        height,
        size.width,
        size.height
    );
}

#[test]
fn test_thumbnail_unsupported_format_error() {
    let temp_dir = TempDir::new().unwrap();
    let unsupported_file = temp_dir.path().join("test.txt");
    fs::write(&unsupported_file, "not an image").unwrap();

    let result = get_thumbnail_bytes(&unsupported_file, ThumbnailSize::default());
    assert!(result.is_err());
}

fn create_test_image(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);

    let img = image::RgbaImage::from_pixel(800, 600, image::Rgba([100, 150, 200, 255]));
    img.save(&path).expect("Should save test image");

    path
}
