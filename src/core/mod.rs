//! Core functionality for CompressCLI
//!
//! This module contains the fundamental components that are used throughout
//! the application, including error handling and configuration management.

pub mod config;
pub mod constants;
pub mod error;

pub use config::{Config, ImagePresetConfig, VideoPresetConfig};
pub use constants::*;
pub use error::{CompressError, Result};
