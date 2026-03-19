use std::env;
use std::path::PathBuf;
use thumb_rs::{get_thumbnail, ThumbnailSize};

/// CLI tool to generate thumbnails using thumb-rs.
///
/// # Usage
/// ```
/// thumb <input_file> [options]
///
/// Options:
///   --size WxH    Thumbnail dimensions (default: 256x256)
///   --output PATH Output PNG path (default: <input_stem>_thumb.png)
/// ```
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: thumb <input_file> [--size WxH] [--output PATH]");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);

    if !input_path.exists() {
        eprintln!("Error: File not found: {}", input_path.display());
        std::process::exit(1);
    }

    // Parse --size flag
    let size = if let Some(idx) = args.iter().position(|a| a == "--size") {
        args.get(idx + 1)
            .and_then(|s| parse_size(s))
            .unwrap_or_default()
    } else {
        ThumbnailSize::default()
    };

    // Parse --output flag
    let output_path = if let Some(idx) = args.iter().position(|a| a == "--output") {
        args.get(idx + 1)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_output_path(&input_path))
    } else {
        default_output_path(&input_path)
    };

    // Generate thumbnail
    let thumb = match get_thumbnail(&input_path, size) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Encode to PNG and save
    // TODO: When png feature is added, use thumb.to_png() instead
    let img =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(thumb.width, thumb.height, thumb.rgba)
            .expect("Failed to create image buffer");

    match img.save(&output_path) {
        Ok(_) => println!("Thumbnail saved to: {}", output_path.display()),
        Err(e) => {
            eprintln!("Error saving: {}", e);
            std::process::exit(1);
        }
    }
}

fn default_output_path(input: &PathBuf) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumbnail");
    let parent = input.parent().unwrap_or(std::path::Path::new("."));
    parent.join(format!("{}_thumb.png", stem))
}

fn parse_size(s: &str) -> Option<ThumbnailSize> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let width: u32 = parts[0].parse().ok()?;
        let height: u32 = parts[1].parse().ok()?;
        Some(ThumbnailSize::new(width, height))
    } else {
        None
    }
}
