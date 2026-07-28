//! Command-line argument definitions and parsing
//!
//! This module defines the CLI structure using clap, including all commands,
//! subcommands, and their respective arguments.

use crate::core::constants::DEFAULT_PARALLEL_JOBS;
pub use crate::core::types::{
    AudioCodec, HwAccelMode, ImageFormat, PresetAction, VideoCodec, VideoPreset,
};
use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "compresscli")]
#[command(about = "A powerful CLI tool for video and image compression")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Dry run - show what would be done without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Overwrite existing files
    #[arg(long, global = true)]
    pub overwrite: bool,

    /// Output directory
    #[arg(short, long, global = true, value_hint = ValueHint::DirPath)]
    pub output_dir: Option<PathBuf>,

    /// Custom config file
    #[arg(long, global = true, value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// Disable caching of compressed files
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Enable GPU hardware acceleration (auto-detects hardware encoder)
    #[arg(long, global = true)]
    pub gpu: bool,

    /// Hardware acceleration mode (auto, nvidia, apple, intel, amd, vaapi, disabled)
    #[arg(long, global = true)]
    pub hwaccel: Option<HwAccelMode>,
}

fn parse_jobs(s: &str) -> Result<usize, String> {
    let val: usize = s
        .parse()
        .map_err(|_| "Jobs must be a positive integer".to_string())?;
    if val < 1 {
        Err("Jobs must be at least 1".to_string())
    } else {
        Ok(val)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive step-by-step compression wizard
    Interactive,

    /// Auto-detect input type (video, image, directory) and process automatically
    Auto {
        /// Input file or directory
        #[arg(value_hint = ValueHint::AnyPath)]
        input: PathBuf,

        /// Output file or directory
        #[arg(value_hint = ValueHint::AnyPath)]
        output: Option<PathBuf>,

        /// Compression preset or quality override
        #[arg(short, long)]
        preset: Option<String>,

        /// Convert format (e.g., "webp", "jpeg", "mp4", "webm")
        #[arg(short, long)]
        format: Option<String>,
    },
    /// Compress video files
    Video {
        /// Input video file
        #[arg(value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file (optional, will auto-generate if not provided)
        #[arg(value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,

        /// Compression preset
        #[arg(short, long, default_value = "medium")]
        preset: VideoPreset,

        /// Video codec
        #[arg(long)]
        codec: Option<VideoCodec>,

        /// Constant Rate Factor (0-51, lower = better quality)
        #[arg(long)]
        crf: Option<u8>,

        /// Target bitrate (e.g., "1M", "500K")
        #[arg(long)]
        bitrate: Option<String>,

        /// Target resolution (e.g., "1920x1080", "720p")
        #[arg(long)]
        resolution: Option<String>,

        /// Target framerate
        #[arg(long)]
        fps: Option<f32>,

        /// Audio codec
        #[arg(long)]
        audio_codec: Option<AudioCodec>,

        /// Audio bitrate (e.g., "128K", "256K")
        #[arg(long)]
        audio_bitrate: Option<String>,

        /// Remove audio track
        #[arg(long)]
        no_audio: bool,

        /// Start time for trimming (e.g., "00:01:30")
        #[arg(long)]
        start: Option<String>,

        /// End time for trimming (e.g., "00:05:00")
        #[arg(long)]
        end: Option<String>,

        /// Two-pass encoding for better quality
        #[arg(long)]
        two_pass: bool,
    },

    /// Compress image files
    Image {
        /// Input image file
        #[arg(value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file (optional, will auto-generate if not provided)
        #[arg(value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,

        /// Image quality (1-100)
        #[arg(short, long, default_value = "85")]
        quality: u8,

        /// Output format
        #[arg(short, long)]
        format: Option<ImageFormat>,

        /// Resize to specific dimensions (e.g., "800x600")
        #[arg(long)]
        resize: Option<String>,

        /// Maximum width (maintains aspect ratio)
        #[arg(long)]
        max_width: Option<u32>,

        /// Maximum height (maintains aspect ratio)
        #[arg(long)]
        max_height: Option<u32>,

        /// Enable optimization
        #[arg(long)]
        optimize: bool,

        /// Progressive JPEG
        #[arg(long)]
        progressive: bool,

        /// Lossless compression (where supported)
        #[arg(long)]
        lossless: bool,

        /// Image preset (web, high, lossless)
        #[arg(short, long)]
        preset: Option<String>,
    },

    /// Batch process files in a directory
    Batch {
        /// Input directory
        #[arg(value_hint = ValueHint::DirPath)]
        directory: PathBuf,

        /// File pattern (e.g., "*.mp4", "*.jpg")
        #[arg(short, long, default_value = "*")]
        pattern: String,

        /// Process videos
        #[arg(long)]
        videos: bool,

        /// Process images
        #[arg(long)]
        images: bool,

        /// Recursive processing
        #[arg(short, long)]
        recursive: bool,

        /// Video preset for batch processing
        #[arg(long, default_value = "medium")]
        video_preset: VideoPreset,

        /// Image preset for batch processing
        #[arg(long)]
        image_preset: Option<String>,

        /// Image quality for batch processing
        #[arg(long, default_value = "85")]
        image_quality: u8,

        /// Maximum parallel jobs
        #[arg(short, long, default_value_t = DEFAULT_PARALLEL_JOBS, value_parser = parse_jobs)]
        jobs: usize,
    },

    /// Manage compression presets
    Presets {
        #[command(subcommand)]
        action: PresetAction,
    },

    /// Show system information and dependencies
    Info,

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_interactive_mode() {
        let cli = Cli::parse_from(["compresscli"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_explicit_interactive_subcommand() {
        let cli = Cli::parse_from(["compresscli", "interactive"]);
        assert!(matches!(cli.command, Some(Commands::Interactive)));
    }

    #[test]
    fn test_cli_batch_image_preset() {
        let cli = Cli::parse_from([
            "compresscli",
            "batch",
            "./images",
            "--images",
            "--image-preset",
            "web",
        ]);
        if let Some(Commands::Batch {
            directory,
            images,
            image_preset,
            ..
        }) = cli.command
        {
            assert_eq!(directory, PathBuf::from("./images"));
            assert!(images);
            assert_eq!(image_preset, Some("web".to_string()));
        } else {
            panic!("Expected Commands::Batch variant");
        }
    }
}
