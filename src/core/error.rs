//! Error handling for CompressCLI

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),


    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid input file: {path}")]
    InvalidInput { path: PathBuf },

    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Missing dependency: {dependency}")]
    MissingDependency { dependency: String },

    #[error("Invalid parameter: {parameter} = {value}")]
    InvalidParameter { parameter: String, value: String },

    #[error("File already exists: {path}")]
    FileExists { path: PathBuf },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Process execution failed: {command}")]
    ProcessFailed { command: String },
}

pub type Result<T> = std::result::Result<T, CompressError>;

impl CompressError {
    /// Creates an error for invalid input file paths
    /// This is used when a file doesn't exist or isn't accessible
    pub fn invalid_input<P: AsRef<Path>>(path: P) -> Self {
        Self::InvalidInput {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Creates an error when trying to overwrite an existing file without permission
    /// Used when output file exists and --overwrite flag is not set
    pub fn file_exists<P: AsRef<Path>>(path: P) -> Self {
        Self::FileExists {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Creates an error for invalid command-line parameters
    /// Used when user provides invalid values for options like quality, resolution, etc.
    pub fn invalid_parameter<S: Into<String>>(param: &str, value: S) -> Self {
        Self::InvalidParameter {
            parameter: param.to_string(),
            value: value.into(),
        }
    }

    /// Creates an error when a required system dependency is missing
    /// Primarily used for FFmpeg dependency checking
    pub fn missing_dependency<S: Into<String>>(dependency: S) -> Self {
        Self::MissingDependency {
            dependency: dependency.into(),
        }
    }

    /// Creates an error for unsupported file formats or codecs
    /// Used when trying to use formats not supported by the system
    pub fn unsupported_format<S: Into<String>>(format: S) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Creates an error for configuration-related issues
    /// Used for config file parsing errors, preset issues, etc.
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Creates an error when external process execution fails
    /// Used when FFmpeg or other external commands fail to execute
    pub fn process_failed<S: Into<String>>(command: S) -> Self {
        Self::ProcessFailed {
            command: command.into(),
        }
    }
}
