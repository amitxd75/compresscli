use crate::cli::args::VideoPreset;
use crate::compression::{
    ImageCompressionOptions, ImageCompressor, VideoCompressionOptions, VideoCompressor,
};
use crate::core::{CompressError, Config, Result};
use crate::ui::progress::{create_file_progress_bar, print_header, print_info, print_success};
use crate::utils::{is_image_file, is_video_file};
use glob::Pattern;
use log::warn;
use std::path::PathBuf;
use tokio::task::JoinSet;
use walkdir::WalkDir;

pub struct BatchProcessor {
    config: Config,
    dry_run: bool,
    verbose: bool,
}

#[derive(Debug, Clone)]
pub struct BatchOptions {
    pub directory: PathBuf,
    pub pattern: String,
    pub videos: bool,
    pub images: bool,
    pub recursive: bool,
    pub video_preset: VideoPreset,
    pub image_quality: u8,
    pub jobs: usize,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

impl BatchProcessor {
    /// Creates a new BatchProcessor instance
    /// Initializes with configuration, dry-run mode, and verbosity settings
    pub fn new(config: Config, dry_run: bool, verbose: bool) -> Self {
        Self {
            config,
            dry_run,
            verbose,
        }
    }

    /// Processes all files in a directory according to the batch options
    /// Handles parallel processing, progress tracking, and result aggregation
    /// Returns statistics about the batch processing operation
    pub async fn process_directory(&self, options: BatchOptions) -> Result<BatchResults> {
        print_header(&format!(
            "Batch Processing: {}",
            options.directory.display()
        ));

        // Find all files matching the specified criteria
        let files = self.find_files(&options)?;

        if files.is_empty() {
            print_info("No files found matching the criteria");
            return Ok(BatchResults::default());
        }

        // Separate video and image files
        let (video_files, image_files) = self.separate_files(&files);

        let mut results = BatchResults::default();

        // Process videos if requested
        if options.videos && !video_files.is_empty() {
            print_info(&format!("Processing {} video files...", video_files.len()));
            let video_results = self.process_videos(video_files, &options).await?;
            results.videos = video_results;
        }

        // Process images if requested
        if options.images && !image_files.is_empty() {
            print_info(&format!("Processing {} image files...", image_files.len()));
            let image_results = self.process_images(image_files, &options).await?;
            results.images = image_results;
        }

        self.print_batch_summary(&results);
        Ok(results)
    }

    /// Finds all files in the directory that match the specified criteria
    /// Supports recursive traversal and pattern matching
    /// Filters by file type (video/image) based on options
    fn find_files(&self, options: &BatchOptions) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let pattern = Pattern::new(&options.pattern)
            .map_err(|e| CompressError::invalid_parameter("pattern", e.to_string()))?;

        let walker = if options.recursive {
            WalkDir::new(&options.directory)
        } else {
            WalkDir::new(&options.directory).max_depth(1)
        };

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(filename) = path.file_name()
                && let Some(filename_str) = filename.to_str()
                && pattern.matches(filename_str)
            {
                // Check if it's a video or image file based on what we're processing
                let is_target_file = (options.videos && is_video_file(path))
                    || (options.images && is_image_file(path));

                if is_target_file {
                    files.push(path.to_path_buf());
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Separates files into video and image categories
    /// Returns tuple of (video_files, image_files) for separate processing
    fn separate_files(&self, files: &[PathBuf]) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let mut video_files = Vec::new();
        let mut image_files = Vec::new();

        for file in files {
            if is_video_file(file) {
                video_files.push(file.clone());
            } else if is_image_file(file) {
                image_files.push(file.clone());
            }
        }

        (video_files, image_files)
    }

    async fn process_videos(
        &self,
        files: Vec<PathBuf>,
        options: &BatchOptions,
    ) -> Result<Vec<PathBuf>> {
        let video_compressor =
            VideoCompressor::new(self.config.clone(), self.dry_run, self.verbose);
        let progress = create_file_progress_bar(files.len());

        let mut results = Vec::new();
        let mut tasks = JoinSet::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(options.jobs));

        for file in files {
            let compressor = video_compressor.clone();
            let batch_options = options.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            tasks.spawn(async move {
                let _permit = permit; // Keep permit alive

                let video_options = VideoCompressionOptions {
                    input: file,
                    output: None,
                    preset: batch_options.video_preset,
                    codec: None,
                    crf: None,
                    bitrate: None,
                    resolution: None,
                    fps: None,
                    audio_codec: None,
                    audio_bitrate: None,
                    no_audio: false,
                    start: None,
                    end: None,
                    two_pass: false,
                    output_dir: batch_options.output_dir,
                    overwrite: batch_options.overwrite,
                };

                compressor.compress(video_options).await
            });
        }

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(output_path)) => {
                    results.push(output_path);
                    progress.inc(1);
                }
                Ok(Err(e)) => {
                    warn!("Video compression failed: {}", e);
                    progress.inc(1);
                }
                Err(e) => {
                    warn!("Task failed: {}", e);
                    progress.inc(1);
                }
            }
        }

        progress.finish_and_clear();
        Ok(results)
    }

    async fn process_images(
        &self,
        files: Vec<PathBuf>,
        options: &BatchOptions,
    ) -> Result<Vec<PathBuf>> {
        let image_compressor =
            ImageCompressor::new(self.config.clone(), self.dry_run, self.verbose);
        let progress = create_file_progress_bar(files.len());

        let mut results = Vec::new();
        let mut tasks = JoinSet::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(options.jobs));

        for file in files {
            let compressor = image_compressor.clone();
            let batch_options = options.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            tasks.spawn(async move {
                let _permit = permit; // Keep permit alive

                let image_options = ImageCompressionOptions {
                    input: file,
                    output: None,
                    quality: batch_options.image_quality,
                    format: None,
                    resize: None,
                    max_width: None,
                    max_height: None,
                    optimize: true,
                    progressive: false,
                    lossless: false,
                    preset: None,
                    output_dir: batch_options.output_dir,
                    overwrite: batch_options.overwrite,
                };

                compressor.compress(image_options).await
            });
        }

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(output_path)) => {
                    results.push(output_path);
                    progress.inc(1);
                }
                Ok(Err(e)) => {
                    warn!("Image compression failed: {}", e);
                    progress.inc(1);
                }
                Err(e) => {
                    warn!("Task failed: {}", e);
                    progress.inc(1);
                }
            }
        }

        progress.finish_and_clear();
        Ok(results)
    }



    fn print_batch_summary(&self, results: &BatchResults) {
        print_header("Batch Processing Complete");

        if !results.videos.is_empty() {
            print_success(&format!("Videos processed: {}", results.videos.len()));
        }

        if !results.images.is_empty() {
            print_success(&format!("Images processed: {}", results.images.len()));
        }

        let total = results.videos.len() + results.images.len();
        if total > 0 {
            print_success(&format!("Total files processed: {}", total));
        }
    }
}

#[derive(Debug, Default)]
pub struct BatchResults {
    pub videos: Vec<PathBuf>,
    pub images: Vec<PathBuf>,
}

impl BatchResults {
    pub fn total_files(&self) -> usize {
        self.videos.len() + self.images.len()
    }
}

// Make VideoCompressor cloneable for async processing
impl Clone for VideoCompressor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            dry_run: self.dry_run,
            verbose: self.verbose,
        }
    }
}

// Make ImageCompressor cloneable for async processing
impl Clone for ImageCompressor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            dry_run: self.dry_run,
            verbose: self.verbose,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_files() {
        let config = Config::default();
        let processor = BatchProcessor::new(config, false, false);

        let files = vec![
            PathBuf::from("video.mp4"),
            PathBuf::from("image.jpg"),
            PathBuf::from("another_video.avi"),
            PathBuf::from("another_image.png"),
        ];

        let (videos, images) = processor.separate_files(&files);

        assert_eq!(videos.len(), 2);
        assert_eq!(images.len(), 2);
    }
}
