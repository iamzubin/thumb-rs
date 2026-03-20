use std::env;
use std::path::{Path, PathBuf};
use thumb_rs::{get_thumbnail, ThumbnailScale};

/// CLI tool to generate thumbnails using thumb-rs.
///
/// # Usage
/// ```
/// thumb <input_file> [options]
///
/// Options:
///   --scale N     Size multiplier (default: 1 → 256×256, 2 → 512×512, etc.)
///   --output PATH Output PNG path (default: <input_stem>_thumb.png)
/// ```
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: thumb <input_file> [--scale N] [--output PATH]");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);

    if !input_path.exists() {
        eprintln!("Error: File not found: {}", input_path.display());
        std::process::exit(1);
    }

    // Parse --scale flag
    let scale = if let Some(idx) = args.iter().position(|a| a == "--scale") {
        args.get(idx + 1)
            .and_then(|s| s.parse::<u32>().ok())
            .map(ThumbnailScale)
            .unwrap_or_default()
    } else {
        ThumbnailScale::default()
    };

    // Parse --output flag
    let output_path = if let Some(idx) = args.iter().position(|a| a == "--output") {
        args.get(idx + 1)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_output_path(&input_path))
    } else {
        default_output_path(&input_path)
    };

    use std::time::Instant;

    // Generate thumbnail
    let start = Instant::now();
    let thumb = match get_thumbnail(&input_path, scale) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let duration = start.elapsed();
    println!("Thumbnail generated in: {:?}", duration);

    // Encode to PNG and save
    let start_save = Instant::now();
    let img =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(thumb.width, thumb.height, thumb.rgba)
            .expect("Failed to create image buffer");

    match img.save(&output_path) {
        Ok(_) => {
            let duration_save = start_save.elapsed();
            println!("Thumbnail saved to: {} (took {:?})", output_path.display(), duration_save);
            println!("Total time: {:?}", duration + duration_save);
        }
        Err(e) => {
            eprintln!("Error saving: {}", e);
            std::process::exit(1);
        }
    }
}

fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumbnail");
    let parent = input.parent().unwrap_or(std::path::Path::new("."));
    parent.join(format!("{}_thumb.png", stem))
}
