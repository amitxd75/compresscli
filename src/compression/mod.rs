//! Compression functionality for CompressCLI
//!
//! This module contains all compression-related functionality including
//! video compression, image compression, and batch processing operations.

pub mod batch;
pub mod image;
pub mod video;

// Re-export main compression types
pub use batch::{BatchOptions, BatchProcessor};
pub use image::{ImageCompressionOptions, ImageCompressor};
pub use video::{VideoCompressionOptions, VideoCompressor};
