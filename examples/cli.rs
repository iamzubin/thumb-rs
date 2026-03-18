use std::env;
use std::path::PathBuf;
use thumb_rs::{get_thumbnail, is_supported, ThumbnailSize};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: thumb <input_file> [output_file] [--size WIDTHxHEIGHT]");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);

    if !input_path.exists() {
        eprintln!("Error: File not found: {}", input_path.display());
        std::process::exit(1);
    }

    if !is_supported(&input_path) {
        eprintln!("Error: Unsupported file format");
        std::process::exit(1);
    }

    let mut output_path = args.get(2).map(PathBuf::from);
    if output_path.is_none() {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("thumbnail");
        let parent = input_path.parent().unwrap_or(std::path::Path::new("."));
        output_path = Some(parent.join(format!("{}_thumb.png", stem)));
    }

    let size = if let Some(idx) = args.iter().position(|a| a == "--size") {
        if let Some(size_arg) = args.get(idx + 1) {
            parse_size(size_arg).unwrap_or_default()
        } else {
            ThumbnailSize::default()
        }
    } else {
        ThumbnailSize::default()
    };

    match get_thumbnail(&input_path, output_path.as_ref().unwrap(), size) {
        Ok(_) => println!("Thumbnail saved to: {}", output_path.unwrap().display()),
        Err(e) => {
            eprintln!("Error generating thumbnail: {}", e);
            std::process::exit(1);
        }
    }
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
