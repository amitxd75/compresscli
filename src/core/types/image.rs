//! Image compression types, options, and parameters

use crate::core::error::{CompressError, Result};
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug)]
pub enum ImageFormat {
    /// JPEG format
    Jpeg,
    /// PNG format
    Png,
    /// WebP format
    Webp,
    /// AVIF format (next-gen)
    Avif,
}

impl ImageFormat {
    /// Parses image format string supporting aliases (e.g., jpg/jpeg)
    pub fn parse_from_str(s: &str) -> std::result::Result<Self, String> {
        match s.to_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::Webp),
            "avif" => Ok(ImageFormat::Avif),
            _ => Err(format!("Unsupported image format: {}", s)),
        }
    }
}

impl std::str::FromStr for ImageFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse_from_str(s)
    }
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Jpeg => write!(f, "jpg"),
            ImageFormat::Png => write!(f, "png"),
            ImageFormat::Webp => write!(f, "webp"),
            ImageFormat::Avif => write!(f, "avif"),
        }
    }
}

/// Parameters passed from the CLI for image compression
#[derive(Debug, Clone)]
pub struct ImageCommandParams {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub quality: u8,
    pub format: Option<ImageFormat>,
    pub resize: Option<String>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub optimize: bool,
    pub progressive: bool,
    pub lossless: bool,
    pub preset: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

/// Options configuring image compression engine
#[derive(Debug, Clone)]
pub struct ImageCompressionOptions {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub quality: u8,
    pub format: Option<ImageFormat>,
    pub resize: Option<String>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub optimize: bool,
    pub progressive: bool,
    pub lossless: bool,
    pub preset: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

impl ImageCompressionOptions {
    /// Enforces strict invariants on ImageCompressionOptions
    pub fn validate(&self) -> Result<()> {
        if self.quality == 0 || self.quality > 100 {
            return Err(CompressError::invalid_parameter(
                "quality",
                format!("Quality must be in range 1..=100, got {}", self.quality),
            ));
        }

        if let Some(w) = self.max_width
            && w == 0
        {
            return Err(CompressError::invalid_parameter(
                "max_width",
                "Max width must be greater than 0",
            ));
        }

        if let Some(h) = self.max_height
            && h == 0
        {
            return Err(CompressError::invalid_parameter(
                "max_height",
                "Max height must be greater than 0",
            ));
        }

        Ok(())
    }

    /// Computes a deterministic content hash of image options (excluding file paths)
    pub fn options_hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.quality.hash(&mut hasher);
        format!("{:?}", self.format).hash(&mut hasher);
        self.resize.hash(&mut hasher);
        self.max_width.hash(&mut hasher);
        self.max_height.hash(&mut hasher);
        self.optimize.hash(&mut hasher);
        self.progressive.hash(&mut hasher);
        self.lossless.hash(&mut hasher);
        self.preset.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl From<ImageCommandParams> for ImageCompressionOptions {
    fn from(p: ImageCommandParams) -> Self {
        Self {
            input: p.input,
            output: p.output,
            quality: p.quality,
            format: p.format,
            resize: p.resize,
            max_width: p.max_width,
            max_height: p.max_height,
            optimize: p.optimize,
            progressive: p.progressive,
            lossless: p.lossless,
            preset: p.preset,
            output_dir: p.output_dir,
            overwrite: p.overwrite,
        }
    }
}
