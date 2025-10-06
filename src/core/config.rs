use crate::cli::args::{AudioCodec, VideoCodec, VideoPreset};
use crate::core::constants::*;
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

        // Default video presets using constants
        let video_preset_configs = [
            (
                "ultrafast",
                VideoCodec::H264,
                CRF_ULTRAFAST,
                AUDIO_BITRATE_LOW,
                false,
            ),
            ("fast", VideoCodec::H264, CRF_FAST, AUDIO_BITRATE_LOW, false),
            (
                "medium",
                VideoCodec::H264,
                CRF_MEDIUM,
                AUDIO_BITRATE_LOW,
                false,
            ),
            (
                "slow",
                VideoCodec::H264,
                CRF_SLOW,
                AUDIO_BITRATE_MEDIUM,
                true,
            ),
            (
                "veryslow",
                VideoCodec::H265,
                CRF_VERYSLOW,
                AUDIO_BITRATE_HIGH,
                true,
            ),
        ];

        for (name, codec, crf, audio_bitrate, two_pass) in video_preset_configs {
            video_presets.insert(
                name.to_string(),
                VideoPresetConfig {
                    codec,
                    crf: Some(crf),
                    bitrate: None,
                    audio_codec: AudioCodec::Aac,
                    audio_bitrate: Some(audio_bitrate.to_string()),
                    preset: name.to_string(),
                    two_pass,
                    extra_args: vec![],
                },
            );
        }

        // Default image presets
        let image_preset_configs = [
            ("web", DEFAULT_IMAGE_QUALITY, true, true, false),
            ("high", 95, true, false, false),
            ("lossless", 100, true, false, true),
        ];

        for (name, quality, optimize, progressive, lossless) in image_preset_configs {
            image_presets.insert(
                name.to_string(),
                ImagePresetConfig {
                    quality,
                    optimize,
                    progressive,
                    lossless,
                },
            );
        }

        Self {
            video_presets,
            image_presets,
            default_settings: DefaultSettings {
                output_dir: None,
                overwrite: false,
                parallel_jobs: num_cpus::get().max(1), // Ensure at least 1 job
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
