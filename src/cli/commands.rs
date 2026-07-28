//! Command execution logic
//!
//! This module contains the core logic for executing different CLI commands
//! including video compression, image compression, batch processing, etc.

use crate::compression::{BatchProcessor, ImageCompressor, VideoCompressor};
pub use crate::core::types::{BatchCommandParams, ImageCommandParams, VideoCommandParams};
use crate::core::{CompressError, Config, Result};
use crate::ui::progress::{print_error, print_success};
use crate::utils;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;

/// Handles video compression command
pub async fn handle_video_command(
    params: VideoCommandParams,
    config: Config,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    check_ffmpeg_dependency()?;

    let compressor = VideoCompressor::new(config, dry_run, verbose);
    let output_path = compressor.compress(params.into()).await?;

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
    let compressor = ImageCompressor::new(config, dry_run, verbose);
    let output_path = compressor.compress(params.into()).await?;

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
    if !params.videos && !params.images {
        return Err(CompressError::config(
            "Must specify --videos and/or --images for batch processing",
        ));
    }

    if params.videos {
        check_ffmpeg_dependency()?;
    }

    let processor = BatchProcessor::new(config, dry_run, verbose);
    let results = processor.process_directory(params.into()).await?;

    if !dry_run && results.total_files() > 0 {
        print_success(&format!(
            "Batch processing complete: {}/{} files processed successfully",
            results.successful_files(),
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
    println!("Rust MSRV: {}", env!("CARGO_PKG_RUST_VERSION"));

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

    let gpu_encoders = utils::system::detect_available_gpu_encoders();
    if gpu_encoders.is_empty() {
        println!("GPU Hardware Acceleration: None detected");
    } else {
        print_success(&format!(
            "GPU Hardware Encoders: {}",
            gpu_encoders.join(", ")
        ));
    }

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
