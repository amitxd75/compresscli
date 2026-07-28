//! File utilities for handling file operations and validation

use crate::core::error::{CompressError, Result};
use crate::core::{IMAGE_EXTENSIONS, VIDEO_EXTENSIONS};
use bytesize::ByteSize;
use std::ffi::OsStr;
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

    let default_ext = if !input_extension.is_empty() {
        input_extension.to_str().unwrap_or("compressed")
    } else {
        "compressed"
    };

    let final_extension = extension.unwrap_or(default_ext);

    if !final_extension.is_empty() {
        filename.push('.');
        filename.push_str(final_extension);
    }

    let output_dir = output_dir.unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")));
    output_dir.join(filename)
}

/// Validates that input file exists and is readable
/// Checks file existence, type, and accessibility before processing
pub fn validate_input_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.try_exists()? {
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
/// Uses try_exists to propagate I/O errors and verify file presence.
pub fn check_output_overwrite<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<()> {
    let path = path.as_ref();

    if path.try_exists()? && !overwrite {
        return Err(CompressError::file_exists(path));
    }

    Ok(())
}

/// Gets list of supported video file extensions
pub fn get_video_extensions() -> Vec<&'static str> {
    VIDEO_EXTENSIONS.to_vec()
}

/// Gets list of supported image file extensions
pub fn get_image_extensions() -> Vec<&'static str> {
    IMAGE_EXTENSIONS.to_vec()
}

/// Checks if a file is a video based on its extension
/// Used for filtering files in batch processing operations
pub fn is_video_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(ext) = get_extension_lowercase(path) {
        VIDEO_EXTENSIONS.contains(&ext.as_str())
    } else {
        false
    }
}

/// Checks if a file is an image based on its extension
/// Used for filtering files in batch processing operations
pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(ext) = get_extension_lowercase(path) {
        IMAGE_EXTENSIONS.contains(&ext.as_str())
    } else {
        false
    }
}

/// Validates that a path is safe for execution
/// Checks for null bytes and tracks component depth to prevent parent directory boundary escapes
pub fn validate_safe_path<P: AsRef<Path>>(path: P) -> Result<()> {
    let p = path.as_ref();
    let path_str = p.to_string_lossy();

    // Check for null bytes
    if path_str.contains('\0') {
        return Err(CompressError::invalid_parameter(
            "path",
            "Null bytes not allowed in paths",
        ));
    }

    // Track relative directory depth to allow safe internal '..' references while blocking boundary escapes
    let mut depth: i32 = 0;
    for component in p.components() {
        match component {
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(CompressError::invalid_parameter(
                        "path",
                        "Path escapes working directory boundary via parent traversal",
                    ));
                }
            }
            std::path::Component::Normal(_) => {
                depth += 1;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Creates parent directories for a file path if they don't exist
/// Returns error if directory creation fails
pub fn ensure_parent_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(CompressError::Io)?;
    }
    Ok(())
}

/// Gets the file extension as a lowercase string
/// Returns None if the file has no extension
pub fn get_extension_lowercase<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_path() {
        // Valid paths
        assert!(validate_safe_path("/valid/path").is_ok());
        assert!(validate_safe_path("relative/path").is_ok());
        assert!(validate_safe_path("safe/../path").is_ok());

        // Invalid paths
        assert!(validate_safe_path("../dangerous").is_err());
        assert!(validate_safe_path("path\0null").is_err());
    }

    #[test]
    fn test_get_extension_lowercase() {
        assert_eq!(get_extension_lowercase("file.TXT"), Some("txt".to_string()));
        assert_eq!(get_extension_lowercase("file.MP4"), Some("mp4".to_string()));
        assert_eq!(get_extension_lowercase("no_extension"), None);
    }

    #[test]
    fn test_file_type_detection() {
        assert!(is_video_file("test.mp4"));
        assert!(is_video_file("test.MP4"));
        assert!(is_image_file("test.jpg"));
        assert!(is_image_file("test.PNG"));
        assert!(!is_video_file("test.txt"));
        assert!(!is_image_file("test.txt"));
    }
}
