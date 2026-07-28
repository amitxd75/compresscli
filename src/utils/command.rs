use crate::core::types::{AudioCodec, HwAccelMode, VideoCodec};
use crate::core::{CompressError, NULL_DEVICE, Result};
use crate::utils::{parse_resolution, parse_time, validate_safe_path};
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

    /// Adds input file with path validation
    pub fn input<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        validate_safe_path(&path)?;
        self.command.arg("-i").arg(path.as_ref());
        Ok(self)
    }

    /// Adds output file with path validation
    pub fn output<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        validate_safe_path(&path)?;
        self.command.arg(path.as_ref());
        Ok(self)
    }

    /// Sets thread allocation for FFmpeg process
    pub fn threads(mut self, threads: usize) -> Self {
        if threads > 0 {
            self.command.arg("-threads").arg(threads.to_string());
        }
        self
    }

    /// Sets video codec
    pub fn video_codec(mut self, codec: VideoCodec) -> Self {
        self.command.arg("-c:v").arg(codec.to_string());
        self
    }

    /// Sets hardware video codec based on HwAccelMode and VideoCodec
    pub fn hardware_video_codec(mut self, codec: &VideoCodec, mode: &HwAccelMode) -> Self {
        let available = crate::utils::system::detect_available_gpu_encoders();

        let hw_encoder_name = match mode {
            HwAccelMode::Nvidia => match codec {
                VideoCodec::H264 => "h264_nvenc",
                VideoCodec::H265 => "hevc_nvenc",
                VideoCodec::Av1 => "av1_nvenc",
                _ => "h264_nvenc",
            },
            HwAccelMode::Apple => match codec {
                VideoCodec::H264 => "h264_videotoolbox",
                VideoCodec::H265 => "hevc_videotoolbox",
                _ => "h264_videotoolbox",
            },
            HwAccelMode::Intel => match codec {
                VideoCodec::H264 => "h264_qsv",
                VideoCodec::H265 => "hevc_qsv",
                _ => "h264_qsv",
            },
            HwAccelMode::Amd => match codec {
                VideoCodec::H264 => "h264_amf",
                VideoCodec::H265 => "hevc_amf",
                _ => "h264_amf",
            },
            HwAccelMode::Vaapi => match codec {
                VideoCodec::H264 => "h264_vaapi",
                VideoCodec::H265 => "hevc_vaapi",
                _ => "h264_vaapi",
            },
            HwAccelMode::Auto => {
                // Auto-pick best detected matching encoder
                let target_prefix = match codec {
                    VideoCodec::H264 => "h264_",
                    VideoCodec::H265 => "hevc_",
                    VideoCodec::Av1 => "av1_",
                    VideoCodec::Vp9 => "vp9_",
                };
                available
                    .iter()
                    .find(|enc| enc.starts_with(target_prefix))
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| match codec {
                        VideoCodec::H264 => "h264_nvenc",
                        VideoCodec::H265 => "hevc_nvenc",
                        _ => "h264_nvenc",
                    })
            }
            HwAccelMode::Disabled => return self.video_codec(codec.clone()),
        };

        if available.contains(&hw_encoder_name.to_string())
            || matches!(mode, HwAccelMode::Nvidia | HwAccelMode::Apple)
        {
            log::info!("Using GPU hardware encoder: {}", hw_encoder_name);
            self.command.arg("-c:v").arg(hw_encoder_name);
        } else {
            log::warn!(
                "Hardware encoder '{}' not verified on system, falling back to software codec {}",
                hw_encoder_name,
                codec
            );
            self.command.arg("-c:v").arg(codec.to_string());
        }

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

    /// Sets encoding preset with codec compatibility checks
    pub fn preset_for_codec(self, preset: &str, codec: &VideoCodec) -> Self {
        if preset.is_empty() {
            return self;
        }

        match codec {
            VideoCodec::H264 | VideoCodec::H265 => self.preset(preset),
            VideoCodec::Vp9 => {
                // Map presets to VP9 cpu-used values
                let cpu_used = match preset {
                    "ultrafast" | "fast" => "5",
                    "medium" => "2",
                    "slow" | "veryslow" => "0",
                    _ => "2",
                };
                let mut s = self;
                s.command.arg("-cpu-used").arg(cpu_used);
                s
            }
            VideoCodec::Av1 => {
                let cpu_used = match preset {
                    "ultrafast" | "fast" => "8",
                    "medium" => "5",
                    "slow" | "veryslow" => "3",
                    _ => "5",
                };
                let mut s = self;
                s.command.arg("-cpu-used").arg(cpu_used);
                s
            }
        }
    }

    /// Sets encoding preset (backward-compatible method)
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
        if fps <= 0.0 || fps > crate::core::constants::MAX_FPS as f32 {
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

    /// Adds custom arguments safely, splitting multi-word argument strings and validating against unsafe arguments.
    pub fn custom_args<I, S>(mut self, args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            let str_val = arg.as_ref();
            if str_val.starts_with("http://") || str_val.starts_with("https://") {
                return Err(CompressError::invalid_parameter(
                    "extra_args",
                    "Network protocol URLs are not allowed in custom arguments",
                ));
            }
            // Split multi-word arguments if space-separated
            for part in str_val.split_whitespace() {
                self.command.arg(part);
            }
        }
        Ok(self)
    }

    /// Builds the final std command
    pub fn build(self) -> Command {
        self.command
    }

    /// Builds tokio process command for non-blocking async execution
    pub fn build_tokio(self) -> tokio::process::Command {
        tokio::process::Command::from(self.build())
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
        self.command.arg("-i").arg(path.as_ref());
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

        let result = FFmpegCommandBuilder::new().framerate(2000.0);
        assert!(result.is_err());

        let result = FFmpegCommandBuilder::new().framerate(144.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hardware_video_codec() {
        use crate::cli::args::HwAccelMode;

        let cmd = FFmpegCommandBuilder::new()
            .input("input.mp4")
            .unwrap()
            .hardware_video_codec(&VideoCodec::H264, &HwAccelMode::Nvidia)
            .build();

        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("h264_nvenc") || cmd_str.contains("libx264"));
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
