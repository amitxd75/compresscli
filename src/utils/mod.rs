//! Utility modules for CompressCLI
//!
//! This module contains various utility functions organized by their purpose:
//! - `system`: System-related utilities (dependency checking, etc.)
//! - `file`: File operations and validation
//! - `parser`: Parsing utilities for various input formats
//! - `math`: Mathematical calculations

pub mod command;
pub mod file;
pub mod math;
pub mod parser;
pub mod progress;
pub mod system;

pub use command::{FFmpegCommandBuilder, FFprobeCommandBuilder};
pub use file::{
    check_output_overwrite, ensure_parent_dir, generate_output_path, get_extension_lowercase,
    get_file_size, get_image_extensions, get_video_extensions, is_image_file, is_video_file,
    quote_path, validate_input_file, validate_safe_path,
};
pub use math::calculate_compression_ratio;
pub use parser::{parse_resolution, parse_time};
pub use progress::{FFmpegProgressParser, ProgressManager, monitor_ffmpeg_progress};
pub use system::{check_command_available, check_ffmpeg};
