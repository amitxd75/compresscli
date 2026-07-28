//! Video compression types, options, and parameters

use crate::core::error::{CompressError, Result};
use crate::core::types::common::HwAccelMode;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
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

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
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

/// Parameters passed from the CLI for video compression
#[derive(Debug, Clone)]
pub struct VideoCommandParams {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub preset: VideoPreset,
    pub codec: Option<VideoCodec>,
    pub crf: Option<u8>,
    pub bitrate: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<f32>,
    pub audio_codec: Option<AudioCodec>,
    pub audio_bitrate: Option<String>,
    pub no_audio: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub two_pass: bool,
    pub hwaccel: Option<HwAccelMode>,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

/// Options configuring video compression engine
#[derive(Debug, Clone)]
pub struct VideoCompressionOptions {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub preset: VideoPreset,
    pub codec: Option<VideoCodec>,
    pub crf: Option<u8>,
    pub bitrate: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<f32>,
    pub audio_codec: Option<AudioCodec>,
    pub audio_bitrate: Option<String>,
    pub no_audio: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub two_pass: bool,
    pub threads: Option<usize>,
    pub hwaccel: Option<HwAccelMode>,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

impl VideoCompressionOptions {
    /// Enforces strict validation invariants on VideoCompressionOptions (CRF bounds, etc.)
    pub fn validate(&self) -> Result<()> {
        if let Some(crf) = self.crf
            && crf > 51
        {
            return Err(CompressError::invalid_parameter(
                "crf",
                format!("CRF must be between 0 and 51, got {}", crf),
            ));
        }
        Ok(())
    }

    /// Enforces strict validation invariants using specified max_fps ceiling
    pub fn validate_with_max_fps(&self, max_fps: f64) -> Result<()> {
        self.validate()?;

        if let Some(fps) = self.fps
            && (fps <= 0.0 || (fps as f64) > max_fps || !fps.is_finite())
        {
            return Err(CompressError::invalid_parameter(
                "fps",
                format!(
                    "Framerate must be positive finite <= {}, got {}",
                    max_fps, fps
                ),
            ));
        }

        Ok(())
    }

    /// Computes a deterministic content hash of the compression options (excluding file paths)
    pub fn options_hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.preset.to_string().hash(&mut hasher);
        format!("{:?}", self.codec).hash(&mut hasher);
        self.crf.hash(&mut hasher);
        self.bitrate.hash(&mut hasher);
        self.resolution.hash(&mut hasher);
        self.fps.map(|f| f.to_bits()).hash(&mut hasher);
        format!("{:?}", self.audio_codec).hash(&mut hasher);
        self.audio_bitrate.hash(&mut hasher);
        self.no_audio.hash(&mut hasher);
        self.start.hash(&mut hasher);
        self.end.hash(&mut hasher);
        self.two_pass.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl From<VideoCommandParams> for VideoCompressionOptions {
    fn from(p: VideoCommandParams) -> Self {
        Self {
            input: p.input,
            output: p.output,
            preset: p.preset,
            codec: p.codec,
            crf: p.crf,
            bitrate: p.bitrate,
            resolution: p.resolution,
            fps: p.fps,
            audio_codec: p.audio_codec,
            audio_bitrate: p.audio_bitrate,
            no_audio: p.no_audio,
            start: p.start,
            end: p.end,
            two_pass: p.two_pass,
            threads: None,
            hwaccel: p.hwaccel,
            output_dir: p.output_dir,
            overwrite: p.overwrite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_compression_options_validate() {
        let opts = VideoCompressionOptions {
            input: PathBuf::from("input.mp4"),
            output: None,
            preset: VideoPreset::Medium,
            codec: None,
            crf: Some(23),
            bitrate: None,
            resolution: None,
            fps: Some(60.0),
            audio_codec: Some(AudioCodec::Aac),
            audio_bitrate: None,
            no_audio: false,
            start: None,
            end: None,
            two_pass: false,
            threads: None,
            hwaccel: None,
            output_dir: None,
            overwrite: false,
        };
        assert!(opts.validate().is_ok());

        let invalid_opts = VideoCompressionOptions {
            crf: Some(52),
            ..opts
        };
        assert!(invalid_opts.validate().is_err());
    }
}
