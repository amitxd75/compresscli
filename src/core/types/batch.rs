//! Batch processing types, options, and parameters

use crate::core::types::video::VideoPreset;
use std::path::PathBuf;

/// Parameters passed from the CLI for batch processing
#[derive(Debug, Clone)]
pub struct BatchCommandParams {
    pub directory: PathBuf,
    pub pattern: String,
    pub videos: bool,
    pub images: bool,
    pub recursive: bool,
    pub video_preset: VideoPreset,
    pub image_preset: Option<String>,
    pub image_quality: u8,
    pub jobs: usize,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

/// Options configuring batch processing engine
#[derive(Debug, Clone)]
pub struct BatchOptions {
    pub directory: PathBuf,
    pub pattern: String,
    pub videos: bool,
    pub images: bool,
    pub recursive: bool,
    pub video_preset: VideoPreset,
    pub image_preset: Option<String>,
    pub image_quality: u8,
    pub jobs: usize,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

impl From<BatchCommandParams> for BatchOptions {
    fn from(p: BatchCommandParams) -> Self {
        Self {
            directory: p.directory,
            pattern: p.pattern,
            videos: p.videos,
            images: p.images,
            recursive: p.recursive,
            video_preset: p.video_preset,
            image_preset: p.image_preset,
            image_quality: p.image_quality,
            jobs: p.jobs,
            output_dir: p.output_dir,
            overwrite: p.overwrite,
        }
    }
}

/// Results of processing a batch of files
#[derive(Debug, Default)]
pub struct BatchResults {
    pub videos: Vec<PathBuf>,
    pub images: Vec<PathBuf>,
    pub failed_videos: Vec<PathBuf>,
    pub failed_images: Vec<PathBuf>,
    pub failure_reasons: Vec<(PathBuf, String)>,
}

impl BatchResults {
    /// Returns the total number of successfully processed files
    pub fn successful_files(&self) -> usize {
        self.videos.len() + self.images.len()
    }

    /// Returns the total number of attempted files (successful + failed)
    pub fn total_files(&self) -> usize {
        self.videos.len() + self.images.len() + self.failed_videos.len() + self.failed_images.len()
    }
}
