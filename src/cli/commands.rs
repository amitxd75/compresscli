//! Command execution logic
//!
//! This module contains the core logic for executing different CLI commands
//! including video compression, image compression, batch processing, etc.

use crate::compression::{
    BatchOptions, BatchProcessor, ImageCompressionOptions, ImageCompressor,
    VideoCompressionOptions, VideoCompressor,
};
use crate::core::{CompressError, Config, Result};
use crate::ui::progress::{print_error, print_success};
use crate::utils;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;

/// Parameters for video compression command
pub struct VideoCommandParams {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub preset: crate::cli::args::VideoPreset,
    pub codec: Option<crate::cli::args::VideoCodec>,
    pub crf: Option<u8>,
    pub bitrate: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<f32>,
    pub audio_codec: Option<crate::cli::args::AudioCodec>,
    pub audio_bitrate: Option<String>,
    pub no_audio: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub two_pass: bool,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

/// Parameters for image compression command
pub struct ImageCommandParams {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub quality: u8,
    pub format: Option<crate::cli::args::ImageFormat>,
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

/// Parameters for batch processing command
pub struct BatchCommandParams {
    pub directory: PathBuf,
    pub pattern: String,
    pub videos: bool,
    pub images: bool,
    pub recursive: bool,
    pub video_preset: crate::cli::args::VideoPreset,
    pub image_quality: u8,
    pub jobs: usize,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

/// Handles video compression command
pub async fn handle_video_command(
    params: VideoCommandParams,
    config: Config,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Check FFmpeg availability
    check_ffmpeg_dependency()?;

    let options = VideoCompressionOptions {
        input: params.input,
        output: params.output,
        preset: params.preset,
        codec: params.codec,
        crf: params.crf,
        bitrate: params.bitrate,
        resolution: params.resolution,
        fps: params.fps,
        audio_codec: params.audio_codec,
        audio_bitrate: params.audio_bitrate,
        no_audio: params.no_audio,
        start: params.start,
        end: params.end,
        two_pass: params.two_pass,
        output_dir: params.output_dir,
        overwrite: params.overwrite,
    };

    let compressor = VideoCompressor::new(config, dry_run, verbose);
    let output_path = compressor.compress(options).await?;

    if !dry_run {
        print_success(&format!("Video saved to: {}", output_path.display()));
    }

    Ok(())
}

/// Handles image compression command
pub async fn handle_image_command(
    params: ImageCommandParams,
    config: Config,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let options = ImageCompressionOptions {
        input: params.input,
        output: params.output,
        quality: params.quality,
        format: params.format,
        resize: params.resize,
        max_width: params.max_width,
        max_height: params.max_height,
        optimize: params.optimize,
        progressive: params.progressive,
        lossless: params.lossless,
        preset: params.preset,
        output_dir: params.output_dir,
        overwrite: params.overwrite,
    };

    let compressor = ImageCompressor::new(config, dry_run, verbose);
    let output_path = compressor.compress(options).await?;

    if !dry_run {
        print_success(&format!("Image saved to: {}", output_path.display()));
    }

    Ok(())
}

/// Handles batch processing command
pub async fn handle_batch_command(
    params: BatchCommandParams,
    config: Config,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Validate that at least one type is selected
    if !params.videos && !params.images {
        return Err(CompressError::config(
            "Must specify --videos and/or --images for batch processing",
        ));
    }

    // Check dependencies based on what we're processing
    if params.videos {
        check_ffmpeg_dependency()?;
    }

    let options = BatchOptions {
        directory: params.directory,
        pattern: params.pattern,
        videos: params.videos,
        images: params.images,
        recursive: params.recursive,
        video_preset: params.video_preset,
        image_quality: params.image_quality,
        jobs: params.jobs,
        output_dir: params.output_dir,
        overwrite: params.overwrite,
    };

    let processor = BatchProcessor::new(config, dry_run, verbose);
    let results = processor.process_directory(options).await?;

    if !dry_run && results.total_files() > 0 {
        print_success(&format!(
            "Batch processing complete: {} files processed",
            results.total_files()
        ));
    }

    Ok(())
}

/// Handles system info command
pub async fn handle_info_command() -> Result<()> {
    use crate::ui::progress::{print_header, print_separator};

    print_header("System Information");

    // Application info
    println!("CompressCLI version: {}", env!("CARGO_PKG_VERSION"));
    println!("Built with Rust: {}", env!("CARGO_PKG_RUST_VERSION"));

    print_separator();

    // FFmpeg info
    match utils::check_ffmpeg() {
        Ok(version) => {
            print_success(&format!("FFmpeg: {}", version));
        }
        Err(_) => {
            print_error("FFmpeg: Not found or not accessible");
            println!("  Install FFmpeg from: https://ffmpeg.org/download.html");
        }
    }

    // Check ffprobe
    if utils::check_command_available("ffprobe") {
        print_success("FFprobe: Available");
    } else {
        print_error("FFprobe: Not found (usually comes with FFmpeg)");
    }

    print_separator();

    // System info
    println!("CPU cores: {}", num_cpus::get());

    // Config location
    if let Ok(config_dir) = Config::get_config_dir() {
        println!("Config directory: {}", config_dir.display());
    }

    print_separator();

    // Supported formats
    println!(
        "Supported video formats: {}",
        utils::get_video_extensions().join(", ")
    );
    println!(
        "Supported image formats: {}",
        utils::get_image_extensions().join(", ")
    );

    Ok(())
}

/// Generates shell completion scripts
pub fn handle_completions_command(shell: Shell) -> Result<()> {
    let mut cmd = crate::cli::args::Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}

/// Checks if FFmpeg is available in the system PATH
/// Returns error if FFmpeg is not found, as it's required for video processing
fn check_ffmpeg_dependency() -> Result<()> {
    if !utils::check_command_available("ffmpeg") {
        return Err(CompressError::missing_dependency("ffmpeg"));
    }
    Ok(())
}