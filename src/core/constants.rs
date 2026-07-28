//! Constants used throughout CompressCLI
//!
//! This module contains all the magic numbers, default values, and configuration
//! constants used across the application to improve maintainability.

/// Default image quality when no preset is specified
pub const DEFAULT_IMAGE_QUALITY: u8 = 85;

/// Default number of parallel jobs for batch processing
pub const DEFAULT_PARALLEL_JOBS: usize = 4;

/// Progress bar update interval in milliseconds
pub const PROGRESS_UPDATE_INTERVAL_MS: u64 = 100;

/// Default video file extension for output
pub const DEFAULT_VIDEO_EXTENSION: &str = "mp4";

/// Default image file extension for output
pub const DEFAULT_IMAGE_EXTENSION: &str = "jpg";

/// Supported video file extensions (lowercase)
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ogv",
];

/// Supported image file extensions (lowercase)
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tiff", "tga", "gif"];

/// FFmpeg progress parsing patterns
pub const FFMPEG_PROGRESS_TIME_PATTERN: &str = "out_time_ms=";
pub const FFMPEG_PROGRESS_FRAME_PATTERN: &str = "frame=";

/// Cross-platform null device paths
#[cfg(not(windows))]
pub const NULL_DEVICE: &str = "/dev/null";
#[cfg(windows)]
pub const NULL_DEVICE: &str = "NUL";

/// Maximum safety ceiling for frame rate (FPS) threshold (supports high-speed camera footage up to 1000 FPS)
pub const MAX_FPS: f64 = 1000.0;

/// Default CRF values for different quality presets
pub const CRF_ULTRAFAST: u8 = 28;
pub const CRF_FAST: u8 = 25;
pub const CRF_MEDIUM: u8 = 23;
pub const CRF_SLOW: u8 = 20;
pub const CRF_VERYSLOW: u8 = 18;

/// Default audio bitrates for different quality levels
pub const AUDIO_BITRATE_LOW: &str = "128k";
pub const AUDIO_BITRATE_MEDIUM: &str = "192k";
pub const AUDIO_BITRATE_HIGH: &str = "256k";
