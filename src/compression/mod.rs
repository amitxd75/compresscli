//! Compression functionality for CompressCLI
//!
//! This module contains all compression-related functionality including
//! video compression, image compression, and batch processing operations.

pub mod batch;
pub mod image;
pub mod video;

// Re-export main compression engines and domain types
pub use batch::BatchProcessor;
pub use image::ImageCompressor;
pub use video::VideoCompressor;

pub use crate::core::types::{ImageCompressionOptions, VideoCompressionOptions};
