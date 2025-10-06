//! Command building utilities for FFmpeg and other external tools

use crate::cli::args::{AudioCodec, VideoCodec};
use crate::core::{CompressError, NULL_DEVICE, Result};
use crate::utils::{parse_resolution, parse_time, quote_path, validate_safe_path};
use std::path::Path;
use std::process::{Command, Stdio};

/// Builder for constructing FFmpeg commands with proper error handling and validation
pub struct FFmpegCommandBuilder {
    command: Command,
}

impl FFmpegCommandBuilder {
    /// Creates a new FFmpeg command builder
    pub fn new() -> Self {
        let mut command = Command::new("ffmpeg");
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        Self { command }
    }

    /// Adds input file with path validation and quoting
    pub fn input<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        validate_safe_path(&path)?;
        self.command.arg("-i").arg(quote_path(path));
        Ok(self)
    }

    /// Adds output file with path validation and quoting
    pub fn output<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        validate_safe_path(&path)?;
        self.command.arg(quote_path(path));
        Ok(self)
    }

    /// Sets video codec
    pub fn video_codec(mut self, codec: VideoCodec) -> Self {
        self.command.arg("-c:v").arg(codec.to_string());
        self
    }

    /// Sets audio codec
    pub fn audio_codec(mut self, codec: AudioCodec) -> Self {
        self.command.arg("-c:a").arg(codec.to_string());
        self
    }

    /// Sets CRF (Constant Rate Factor) for quality-based encoding
    pub fn crf(mut self, crf: u8) -> Result<Self> {
        if crf > 51 {
            return Err(CompressError::invalid_parameter("crf", crf.to_string()));
        }
        self.command.arg("-crf").arg(crf.to_string());
        Ok(self)
    }

    /// Sets target bitrate
    pub fn bitrate(mut self, bitrate: &str) -> Result<Self> {
        // Basic validation of bitrate format
        if !bitrate.chars().any(|c| c.is_ascii_digit()) {
            return Err(CompressError::invalid_parameter("bitrate", bitrate));
        }
        self.command.arg("-b:v").arg(bitrate);
        Ok(self)
    }

    /// Sets audio bitrate
    pub fn audio_bitrate(mut self, bitrate: &str) -> Result<Self> {
        if !bitrate.chars().any(|c| c.is_ascii_digit()) {
            return Err(CompressError::invalid_parameter("audio_bitrate", bitrate));
        }
        self.command.arg("-b:a").arg(bitrate);
        Ok(self)
    }

    /// Sets encoding preset (ultrafast, fast, medium, slow, veryslow)
    pub fn preset(mut self, preset: &str) -> Self {
        if !preset.is_empty() {
            self.command.arg("-preset").arg(preset);
        }
        self
    }

    /// Sets resolution with validation
    pub fn resolution(mut self, resolution: &str) -> Result<Self> {
        let (width, height) = parse_resolution(resolution)?;
        self.command
            .arg("-vf")
            .arg(format!("scale={}:{}", width, height));
        Ok(self)
    }

    /// Sets frame rate
    pub fn framerate(mut self, fps: f32) -> Result<Self> {
        if fps <= 0.0 || fps > 120.0 {
            return Err(CompressError::invalid_parameter("fps", fps.to_string()));
        }
        self.command.arg("-r").arg(fps.to_string());
        Ok(self)
    }

    /// Sets start time for trimming
    pub fn start_time(mut self, time: &str) -> Result<Self> {
        let seconds = parse_time(time)?;
        self.command.arg("-ss").arg(seconds.to_string());
        Ok(self)
    }

    /// Sets duration for trimming
    pub fn duration(mut self, duration: &str) -> Result<Self> {
        let seconds = parse_time(duration)?;
        self.command.arg("-t").arg(seconds.to_string());
        Ok(self)
    }

    /// Disables audio track
    pub fn no_audio(mut self) -> Self {
        self.command.arg("-an");
        self
    }

    /// Enables progress reporting
    pub fn progress(mut self) -> Self {
        self.command.arg("-progress").arg("pipe:1");
        self
    }

    /// Forces overwrite of output files
    pub fn overwrite(mut self) -> Self {
        self.command.arg("-y");
        self
    }

    /// Sets up for first pass of two-pass encoding
    pub fn first_pass(mut self) -> Self {
        self.command
            .arg("-pass")
            .arg("1")
            .arg("-f")
            .arg("null")
            .arg(NULL_DEVICE);
        self
    }

    /// Sets up for second pass of two-pass encoding
    pub fn second_pass(mut self) -> Self {
        self.command.arg("-pass").arg("2");
        self
    }

    /// Adds custom arguments
    pub fn custom_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.command.arg(arg.as_ref());
        }
        self
    }

    /// Builds the final command
    pub fn build(self) -> Command {
        self.command
    }

    /// Gets a string representation of the command for logging
    #[allow(dead_code)]
    pub fn command_string(&self) -> String {
        format!("{:?}", self.command)
    }
}

impl Default for FFmpegCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing FFprobe commands
pub struct FFprobeCommandBuilder {
    command: Command,
}

impl FFprobeCommandBuilder {
    /// Creates a new FFprobe command builder
    pub fn new() -> Self {
        let mut command = Command::new("ffprobe");
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        Self { command }
    }

    /// Sets input file with validation
    pub fn input<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        validate_safe_path(&path)?;
        self.command.arg(quote_path(path));
        Ok(self)
    }

    /// Gets video duration
    pub fn duration(mut self) -> Self {
        self.command
            .arg("-v")
            .arg("quiet")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("csv=p=0");
        self
    }

    /// Gets video metadata
    #[allow(dead_code)]
    pub fn metadata(mut self) -> Self {
        self.command
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams");
        self
    }

    /// Builds the final command
    pub fn build(self) -> Command {
        self.command
    }
}

impl Default for FFprobeCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{AudioCodec, VideoCodec};

    #[test]
    fn test_ffmpeg_command_builder() {
        let cmd = FFmpegCommandBuilder::new()
            .input("input.mp4")
            .unwrap()
            .output("output.mp4")
            .unwrap()
            .video_codec(VideoCodec::H264)
            .audio_codec(AudioCodec::Aac)
            .crf(23)
            .unwrap()
            .preset("medium")
            .overwrite()
            .build();

        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("input.mp4"));
        assert!(cmd_str.contains("output.mp4"));
        assert!(cmd_str.contains("-c:v"));
        assert!(cmd_str.contains("-c:a"));
        assert!(cmd_str.contains("-crf"));
        assert!(cmd_str.contains("23"));
    }

    #[test]
    fn test_invalid_crf() {
        let result = FFmpegCommandBuilder::new().crf(52);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fps() {
        let result = FFmpegCommandBuilder::new().framerate(-1.0);
        assert!(result.is_err());

        let result = FFmpegCommandBuilder::new().framerate(200.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_ffprobe_builder() {
        let cmd = FFprobeCommandBuilder::new()
            .input("test.mp4")
            .unwrap()
            .duration()
            .build();

        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("test.mp4"));
        assert!(cmd_str.contains("-show_entries"));
        assert!(cmd_str.contains("format=duration"));
    }
}
