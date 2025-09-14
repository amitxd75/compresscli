//! File utilities for handling file operations and validation

use crate::core::error::{CompressError, Result};
use bytesize::ByteSize;
use std::path::{Path, PathBuf};

/// Gets file size in a human-readable format using ByteSize
/// This is used to display file sizes and calculate compression ratios
pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<ByteSize> {
    let metadata = std::fs::metadata(path)?;
    Ok(ByteSize::b(metadata.len()))
}

/// Generates output filename based on input file and options
/// Handles suffix addition, extension changes, and output directory placement
pub fn generate_output_path(
    input: &Path,
    output_dir: Option<&Path>,
    suffix: Option<&str>,
    extension: Option<&str>,
) -> PathBuf {
    let input_stem = input.file_stem().unwrap_or_default();
    let input_extension = input.extension().unwrap_or_default();

    let mut filename = input_stem.to_string_lossy().to_string();

    if let Some(suffix) = suffix {
        filename.push_str(suffix);
    }

    let final_extension = extension.unwrap_or_else(|| input_extension.to_str().unwrap_or("out"));

    filename.push('.');
    filename.push_str(final_extension);

    let output_dir = output_dir.unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")));
    output_dir.join(filename)
}

/// Validates that input file exists and is readable
/// Checks file existence, type, and accessibility before processing
pub fn validate_input_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(CompressError::invalid_input(path));
    }

    if !path.is_file() {
        return Err(CompressError::invalid_input(path));
    }

    // Try to read metadata to check if file is accessible
    std::fs::metadata(path)?;

    Ok(())
}

/// Checks if output file would overwrite existing file without permission
/// Returns error if file exists and overwrite flag is not set
pub fn check_output_overwrite<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<()> {
    let path = path.as_ref();

    if path.exists() && !overwrite {
        return Err(CompressError::file_exists(path));
    }

    Ok(())
}

/// Gets list of supported video file extensions
/// Includes both lowercase and uppercase variants for cross-platform compatibility
pub fn get_video_extensions() -> &'static [&'static str] {
    &[
        "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ogv", "MP4", "AVI", "MKV",
        "MOV", "WMV", "FLV", "WEBM", "M4V", "3GP", "OGV",
    ]
}

/// Gets list of supported image file extensions
/// Includes both lowercase and uppercase variants for cross-platform compatibility
pub fn get_image_extensions() -> &'static [&'static str] {
    &[
        "jpg", "jpeg", "png", "webp", "bmp", "tiff", "tga", "gif", "JPG", "JPEG", "PNG", "WEBP",
        "BMP", "TIFF", "TGA", "GIF",
    ]
}

/// Checks if a file is a video based on its extension
/// Used for filtering files in batch processing operations
pub fn is_video_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(extension) = path.as_ref().extension()
        && let Some(ext_str) = extension.to_str()
    {
        return get_video_extensions().contains(&ext_str);
    }
    false
}

/// Checks if a file is an image based on its extension
/// Used for filtering files in batch processing operations
pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(extension) = path.as_ref().extension()
        && let Some(ext_str) = extension.to_str()
    {
        return get_image_extensions().contains(&ext_str);
    }
    false
}
