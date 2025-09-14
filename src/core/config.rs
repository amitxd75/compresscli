use crate::cli::args::{AudioCodec, VideoCodec, VideoPreset};
use crate::core::error::{CompressError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub video_presets: HashMap<String, VideoPresetConfig>,
    pub image_presets: HashMap<String, ImagePresetConfig>,
    pub default_settings: DefaultSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPresetConfig {
    pub codec: VideoCodec,
    pub crf: Option<u8>,
    pub bitrate: Option<String>,
    pub audio_codec: AudioCodec,
    pub audio_bitrate: Option<String>,
    pub preset: String,
    pub two_pass: bool,
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePresetConfig {
    pub quality: u8,
    pub optimize: bool,
    pub progressive: bool,
    pub lossless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSettings {
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
    pub parallel_jobs: usize,
    pub preserve_metadata: bool,
    pub backup_originals: bool,
}

impl Config {
    /// Creates a new Config instance with default presets
    /// This initializes built-in video and image presets for common use cases
    pub fn default() -> Self {
        let mut video_presets = HashMap::new();
        let mut image_presets = HashMap::new();

        // Default video presets
        video_presets.insert(
            "ultrafast".to_string(),
            VideoPresetConfig {
                codec: VideoCodec::H264,
                crf: Some(28),
                bitrate: None,
                audio_codec: AudioCodec::Aac,
                audio_bitrate: Some("128k".to_string()),
                preset: "ultrafast".to_string(),
                two_pass: false,
                extra_args: vec![],
            },
        );

        video_presets.insert(
            "fast".to_string(),
            VideoPresetConfig {
                codec: VideoCodec::H264,
                crf: Some(25),
                bitrate: None,
                audio_codec: AudioCodec::Aac,
                audio_bitrate: Some("128k".to_string()),
                preset: "fast".to_string(),
                two_pass: false,
                extra_args: vec![],
            },
        );

        video_presets.insert(
            "medium".to_string(),
            VideoPresetConfig {
                codec: VideoCodec::H264,
                crf: Some(23),
                bitrate: None,
                audio_codec: AudioCodec::Aac,
                audio_bitrate: Some("128k".to_string()),
                preset: "medium".to_string(),
                two_pass: false,
                extra_args: vec![],
            },
        );

        video_presets.insert(
            "slow".to_string(),
            VideoPresetConfig {
                codec: VideoCodec::H264,
                crf: Some(20),
                bitrate: None,
                audio_codec: AudioCodec::Aac,
                audio_bitrate: Some("192k".to_string()),
                preset: "slow".to_string(),
                two_pass: true,
                extra_args: vec![],
            },
        );

        video_presets.insert(
            "veryslow".to_string(),
            VideoPresetConfig {
                codec: VideoCodec::H265,
                crf: Some(18),
                bitrate: None,
                audio_codec: AudioCodec::Aac,
                audio_bitrate: Some("256k".to_string()),
                preset: "veryslow".to_string(),
                two_pass: true,
                extra_args: vec![],
            },
        );

        // Default image presets
        image_presets.insert(
            "web".to_string(),
            ImagePresetConfig {
                quality: 85,
                optimize: true,
                progressive: true,
                lossless: false,
            },
        );

        image_presets.insert(
            "high".to_string(),
            ImagePresetConfig {
                quality: 95,
                optimize: true,
                progressive: false,
                lossless: false,
            },
        );

        image_presets.insert(
            "lossless".to_string(),
            ImagePresetConfig {
                quality: 100,
                optimize: true,
                progressive: false,
                lossless: true,
            },
        );

        Self {
            video_presets,
            image_presets,
            default_settings: DefaultSettings {
                output_dir: None,
                overwrite: false,
                parallel_jobs: num_cpus::get(),
                preserve_metadata: true,
                backup_originals: false,
            },
        }
    }

    /// Loads configuration from a YAML or TOML file
    /// Automatically detects file format based on extension
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref)?;
        let config: Config = if path_ref.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };
        Ok(config)
    }

    /// Saves the current configuration to a file
    /// Creates parent directories if they don't exist
    /// Format is determined by file extension (.toml or .yaml/.yml)
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = if path_ref.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(self)?
        } else {
            serde_yaml::to_string(self)?
        };

        fs::write(path_ref, content)?;
        Ok(())
    }

    /// Gets the configuration directory for CompressCLI
    /// Creates the directory if it doesn't exist
    /// Returns platform-specific config directory (e.g., ~/.config/compresscli on Linux)
    pub fn get_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CompressError::config("Could not determine config directory"))?
            .join("compresscli");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir)
    }

    /// Gets the default configuration file path
    /// Returns the path to config.yaml in the config directory
    pub fn get_default_config_path() -> Result<PathBuf> {
        Ok(Self::get_config_dir()?.join("config.yaml"))
    }

    /// Loads configuration from default location or creates it if it doesn't exist
    /// This is the main entry point for configuration loading
    pub fn load_or_create_default() -> Result<Self> {
        let config_path = Self::get_default_config_path()?;

        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            let config = Self::default();
            config.save_to_file(config_path)?;
            Ok(config)
        }
    }

    /// Gets a video preset configuration by preset type
    /// Returns None if the preset doesn't exist
    pub fn get_video_preset(&self, preset: &VideoPreset) -> Option<&VideoPresetConfig> {
        self.video_presets.get(&preset.to_string())
    }

    /// Gets an image preset configuration by name
    /// Returns None if the preset doesn't exist
    pub fn get_image_preset(&self, name: &str) -> Option<&ImagePresetConfig> {
        self.image_presets.get(name)
    }

    /// Adds a new video preset or updates an existing one
    /// This allows users to create custom video compression presets
    pub fn add_video_preset(&mut self, name: String, preset: VideoPresetConfig) {
        self.video_presets.insert(name, preset);
    }

    /// Adds a new image preset or updates an existing one
    /// This allows users to create custom image compression presets
    pub fn add_image_preset(&mut self, name: String, preset: ImagePresetConfig) {
        self.image_presets.insert(name, preset);
    }

    /// Removes a video preset by name
    /// Returns true if the preset existed and was removed
    pub fn remove_video_preset(&mut self, name: &str) -> bool {
        self.video_presets.remove(name).is_some()
    }

    /// Removes an image preset by name
    /// Returns true if the preset existed and was removed
    pub fn remove_image_preset(&mut self, name: &str) -> bool {
        self.image_presets.remove(name).is_some()
    }
}
