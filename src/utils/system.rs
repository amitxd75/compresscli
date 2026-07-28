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

/// Detects available GPU hardware encoders by querying `ffmpeg -encoders`
pub fn detect_available_gpu_encoders() -> Vec<String> {
    if !check_command_available("ffmpeg") {
        return Vec::new();
    }

    let output = match Command::new("ffmpeg").arg("-encoders").output() {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut available = Vec::new();

    let candidate_encoders = [
        "h264_nvenc",
        "hevc_nvenc",
        "av1_nvenc",
        "h264_videotoolbox",
        "hevc_videotoolbox",
        "h264_qsv",
        "hevc_qsv",
        "h264_amf",
        "hevc_amf",
        "h264_vaapi",
        "hevc_vaapi",
    ];

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let encoder_name = parts[1];
            if candidate_encoders.contains(&encoder_name)
                && !available.contains(&encoder_name.to_string())
            {
                available.push(encoder_name.to_string());
            }
        }
    }

    available
}
