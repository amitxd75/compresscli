//! Utility modules for CompressCLI
//!
//! This module contains various utility functions organized by their purpose:
//! - `system`: System-related utilities (dependency checking, etc.)
//! - `file`: File operations and validation
//! - `parser`: Parsing utilities for various input formats
//! - `math`: Mathematical calculations

pub mod file;
pub mod math;
pub mod parser;
pub mod system;

// Re-export commonly used functions for convenience
pub use file::{
    check_output_overwrite, generate_output_path, get_file_size, get_image_extensions,
    get_video_extensions, is_image_file, is_video_file, validate_input_file,
};
pub use math::calculate_compression_ratio;
pub use parser::{parse_resolution, parse_time};
pub use system::{check_command_available, check_ffmpeg};
