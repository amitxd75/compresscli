//! System utilities for checking dependencies and system information

use crate::core::error::{CompressError, Result};
use std::process::Command;

/// Checks if a command is available in the system PATH
/// This is used to verify that external dependencies like FFmpeg are installed
pub fn check_command_available(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Checks if FFmpeg is available and gets version information
/// Returns the first line of FFmpeg version output or an error if not found
pub fn check_ffmpeg() -> Result<String> {
    if !check_command_available("ffmpeg") {
        return Err(CompressError::missing_dependency("ffmpeg"));
    }

    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| CompressError::missing_dependency("ffmpeg"))?;

    let version_info = String::from_utf8_lossy(&output.stdout);
    let first_line = version_info.lines().next().unwrap_or("Unknown version");

    Ok(first_line.to_string())
}
