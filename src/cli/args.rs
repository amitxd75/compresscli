//! Command-line argument definitions and parsing
//!
//! This module defines the CLI structure using clap, including all commands,
//! subcommands, and their respective arguments.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "compresscli")]
#[command(about = "A powerful CLI tool for video and image compression")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

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
    #[arg(short, long, global = true)]
    pub output_dir: Option<PathBuf>,

    /// Custom config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compress video files
    Video {
        /// Input video file
        input: PathBuf,

        /// Output file (optional, will auto-generate if not provided)
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
        input: PathBuf,

        /// Output file (optional, will auto-generate if not provided)
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

        /// Image quality for batch processing
        #[arg(long, default_value = "85")]
        image_quality: u8,

        /// Maximum parallel jobs
        #[arg(short, long, default_value = "4")]
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

#[derive(Subcommand)]
pub enum PresetAction {
    /// List all available presets
    List,

    /// Show details of a specific preset
    Show {
        /// Preset name
        name: String,
    },

    /// Create a custom preset
    Create {
        /// Preset name
        name: String,

        /// Preset configuration file
        config: PathBuf,
    },

    /// Delete a custom preset
    Delete {
        /// Preset name
        name: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum VideoPreset {
    /// Fast compression, larger file size
    Fast,
    /// Balanced compression and quality
    Medium,
    /// Slow compression, smaller file size
    Slow,
    /// Ultra-fast compression
    Ultrafast,
    /// Very slow, maximum compression
    Veryslow,
    /// Custom settings
    Custom,
}

#[derive(ValueEnum, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum VideoCodec {
    /// H.264 (widely compatible)
    H264,
    /// H.265/HEVC (better compression)
    H265,
    /// VP9 (open source)
    Vp9,
    /// AV1 (next-gen codec)
    Av1,
}

#[derive(ValueEnum, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AudioCodec {
    /// AAC (widely compatible)
    Aac,
    /// MP3 (legacy)
    Mp3,
    /// Opus (high quality)
    Opus,
    /// Copy original
    Copy,
}

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

impl std::fmt::Display for VideoPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoPreset::Fast => write!(f, "fast"),
            VideoPreset::Medium => write!(f, "medium"),
            VideoPreset::Slow => write!(f, "slow"),
            VideoPreset::Ultrafast => write!(f, "ultrafast"),
            VideoPreset::Veryslow => write!(f, "veryslow"),
            VideoPreset::Custom => write!(f, "custom"),
        }
    }
}

impl std::fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoCodec::H264 => write!(f, "libx264"),
            VideoCodec::H265 => write!(f, "libx265"),
            VideoCodec::Vp9 => write!(f, "libvpx-vp9"),
            VideoCodec::Av1 => write!(f, "libaom-av1"),
        }
    }
}

impl std::fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioCodec::Aac => write!(f, "aac"),
            AudioCodec::Mp3 => write!(f, "libmp3lame"),
            AudioCodec::Opus => write!(f, "libopus"),
            AudioCodec::Copy => write!(f, "copy"),
        }
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