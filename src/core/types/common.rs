//! Shared common enums across CLI and compression engines

use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HwAccelMode {
    /// Auto-detect available GPU hardware encoder
    Auto,
    /// NVIDIA NVENC (h264_nvenc, hevc_nvenc)
    Nvidia,
    /// Apple VideoToolbox (h264_videotoolbox, hevc_videotoolbox)
    Apple,
    /// Intel QuickSync Video (h264_qsv, hevc_qsv)
    Intel,
    /// AMD AMF (h264_amf, hevc_amf)
    Amd,
    /// Linux VAAPI (h264_vaapi, hevc_vaapi)
    Vaapi,
    /// Disable hardware acceleration
    Disabled,
}

impl std::fmt::Display for HwAccelMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HwAccelMode::Auto => write!(f, "auto"),
            HwAccelMode::Nvidia => write!(f, "nvidia"),
            HwAccelMode::Apple => write!(f, "apple"),
            HwAccelMode::Intel => write!(f, "intel"),
            HwAccelMode::Amd => write!(f, "amd"),
            HwAccelMode::Vaapi => write!(f, "vaapi"),
            HwAccelMode::Disabled => write!(f, "disabled"),
        }
    }
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
