use crate::cli::args::VideoPreset;
use crate::compression::{
    ImageCompressionOptions, ImageCompressor, VideoCompressionOptions, VideoCompressor,
};
use crate::core::{CompressError, Config, Result};
use crate::ui::progress::{print_header, print_info, print_success};
use crate::utils::{ProgressManager, is_image_file, is_video_file};
use glob::Pattern;
use log::{error, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
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
            results.videos = video_results.successful;
            results.failed_videos = video_results.failed;
        }

        // Process images if requested
        if options.images && !image_files.is_empty() {
            print_info(&format!("Processing {} image files...", image_files.len()));
            let image_results = self.process_images(image_files, &options).await?;
            results.images = image_results.successful;
            results.failed_images = image_results.failed;
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

    /// Processes video files with error handling and resource management
    async fn process_videos(
        &self,
        files: Vec<PathBuf>,
        options: &BatchOptions,
    ) -> Result<ProcessingResults> {
        let video_compressor =
            VideoCompressor::new(self.config.clone(), self.dry_run, self.verbose);
        let progress = ProgressManager::new_file_progress(files.len());

        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut tasks: JoinSet<Result<(PathBuf, Option<PathBuf>)>> = JoinSet::new();
        let semaphore = Arc::new(Semaphore::new(options.jobs));

        // Spawn tasks for all files
        for file in files {
            let compressor = video_compressor.clone();
            let batch_options = options.clone();
            let permit = Arc::clone(&semaphore);

            tasks.spawn(async move {
                // Acquire permit at the start of the task
                let _permit = permit.acquire().await.map_err(|e| {
                    CompressError::process_failed(format!("Failed to acquire semaphore: {}", e))
                })?;

                let video_options = VideoCompressionOptions {
                    input: file.clone(),
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

                match compressor.compress(video_options).await {
                    Ok(output_path) => Ok((file, Some(output_path))),
                    Err(_e) => Ok((file, None)),
                }
            });
        }

        // Collect results as tasks complete
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok((_input_file, Some(output_path)))) => {
                    successful.push(output_path);
                    progress.inc(1);
                }
                Ok(Ok((input_file, None))) => {
                    failed.push(input_file);
                    progress.inc(1);
                }
                Ok(Err(e)) => {
                    error!("Video compression task failed: {}", e);
                    progress.inc(1);
                }
                Err(e) => {
                    error!("Task join error: {}", e);
                    progress.inc(1);
                }
            }
        }

        progress.finish_and_clear();
        Ok(ProcessingResults { successful, failed })
    }

    /// Processes image files with error handling and resource management
    async fn process_images(
        &self,
        files: Vec<PathBuf>,
        options: &BatchOptions,
    ) -> Result<ProcessingResults> {
        let image_compressor =
            ImageCompressor::new(self.config.clone(), self.dry_run, self.verbose);
        let progress = ProgressManager::new_file_progress(files.len());

        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut tasks: JoinSet<Result<(PathBuf, Option<PathBuf>)>> = JoinSet::new();
        let semaphore = Arc::new(Semaphore::new(options.jobs));

        // Spawn tasks for all files
        for file in files {
            let compressor = image_compressor.clone();
            let batch_options = options.clone();
            let permit = Arc::clone(&semaphore);

            tasks.spawn(async move {
                // Acquire permit at the start of the task
                let _permit = permit.acquire().await.map_err(|e| {
                    CompressError::process_failed(format!("Failed to acquire semaphore: {}", e))
                })?;

                let image_options = ImageCompressionOptions {
                    input: file.clone(),
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

                match compressor.compress(image_options).await {
                    Ok(output_path) => Ok((file, Some(output_path))),
                    Err(_e) => Ok((file, None)),
                }
            });
        }

        // Collect results as tasks complete
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok((_input_file, Some(output_path)))) => {
                    successful.push(output_path);
                    progress.inc(1);
                }
                Ok(Ok((input_file, None))) => {
                    failed.push(input_file);
                    progress.inc(1);
                }
                Ok(Err(e)) => {
                    error!("Image compression task failed: {}", e);
                    progress.inc(1);
                }
                Err(e) => {
                    error!("Task join error: {}", e);
                    progress.inc(1);
                }
            }
        }

        progress.finish_and_clear();
        Ok(ProcessingResults { successful, failed })
    }

    /// Prints a summary of batch processing results
    fn print_batch_summary(&self, results: &BatchResults) {
        print_header("Batch Processing Complete");

        if !results.videos.is_empty() {
            print_success(&format!("Videos processed: {}", results.videos.len()));
        }
        if !results.failed_videos.is_empty() {
            warn!("Videos failed: {}", results.failed_videos.len());
        }

        if !results.images.is_empty() {
            print_success(&format!("Images processed: {}", results.images.len()));
        }
        if !results.failed_images.is_empty() {
            warn!("Images failed: {}", results.failed_images.len());
        }

        let total_successful = results.videos.len() + results.images.len();
        let total_failed = results.failed_videos.len() + results.failed_images.len();

        if total_successful > 0 {
            print_success(&format!(
                "Total files processed successfully: {}",
                total_successful
            ));
        }
        if total_failed > 0 {
            warn!("Total files failed: {}", total_failed);
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
}

impl BatchResults {
    /// Returns the total number of successfully processed files
    pub fn total_files(&self) -> usize {
        self.videos.len() + self.images.len()
    }
}

/// Internal structure for tracking processing results
#[derive(Debug)]
struct ProcessingResults {
    successful: Vec<PathBuf>,
    failed: Vec<PathBuf>,
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

    #[test]
    fn test_batch_results() {
        let mut results = BatchResults::default();
        results.videos.push(PathBuf::from("output1.mp4"));
        results.images.push(PathBuf::from("output1.jpg"));
        results.failed_videos.push(PathBuf::from("failed.mp4"));

        assert_eq!(results.total_files(), 2);
        assert_eq!(results.failed_videos.len() + results.failed_images.len(), 1);
    }
}
