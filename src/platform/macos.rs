use crate::{error::ThumbsError, ThumbnailSize};
use std::path::Path;

pub fn generate_thumbnail(
    file_path: &Path,
    output_path: &Path,
    size: ThumbnailSize,
) -> Result<(), ThumbsError> {
    let img = image::open(file_path).map_err(|e| ThumbsError::ImageError(e.to_string()))?;
    let thumbnail = img.thumbnail(size.width, size.height);

    thumbnail
        .save(output_path)
        .map_err(|e| ThumbsError::SaveError(e.to_string()))?;

    Ok(())
}
