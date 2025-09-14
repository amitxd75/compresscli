use crate::cli::args::{AudioCodec, VideoCodec, VideoPreset};
use crate::core::{CompressError, Config, Result, VideoPresetConfig};
use crate::ui::progress::{create_compression_progress_bar, print_success};
use crate::utils::{
    calculate_compression_ratio, check_output_overwrite, generate_output_path, get_file_size,
    parse_resolution, parse_time, validate_input_file,
};
use log::{debug, info};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct VideoCompressor {
    pub config: Config,
    pub dry_run: bool,
    pub verbose: bool,
}

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
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

impl VideoCompressor {
    /// Creates a new VideoCompressor instance
    /// Initializes with configuration, dry-run mode, and verbosity settings
    pub fn new(config: Config, dry_run: bool, verbose: bool) -> Self {
        Self {
            config,
            dry_run,
            verbose,
        }
    }

    /// Compresses a video file using the specified options
    /// Handles preset application, FFmpeg command building, and execution
    /// Returns the path to the compressed output file
    pub async fn compress(&self, options: VideoCompressionOptions) -> Result<PathBuf> {
        // Validate input file exists and is accessible
        validate_input_file(&options.input)?;

        // Get video preset configuration from config
        let preset_config = self.get_preset_config(&options)?;

        // Generate output path with appropriate naming
        let output_path = self.generate_output_path(&options)?;

        // Check if we should overwrite existing files
        check_output_overwrite(&output_path, options.overwrite)?;

        // Get original file size
        let original_size = get_file_size(&options.input)?;

        info!(
            "Compressing video: {} -> {}",
            options.input.display(),
            output_path.display()
        );

        if self.dry_run {
            self.print_dry_run_info(&options, &preset_config, &output_path);
            return Ok(output_path);
        }

        // Create progress bar
        let progress = create_compression_progress_bar();
        progress.set_message("Analyzing video...");

        // Get video duration for progress tracking
        let duration = self.get_video_duration(&options.input)?;

        // Build FFmpeg command
        let mut cmd = self.build_ffmpeg_command(&options, &preset_config, &output_path)?;

        progress.set_message("Compressing video...");

        // Execute compression
        if preset_config.two_pass && options.bitrate.is_some() {
            self.execute_two_pass_compression(
                &mut cmd,
                &options,
                &preset_config,
                &output_path,
                duration,
            )
            .await?;
        } else {
            self.execute_single_pass_compression(&mut cmd, duration)
                .await?;
        }

        progress.finish_and_clear();

        // Get compressed file size and calculate ratio
        let compressed_size = get_file_size(&output_path)?;
        let compression_ratio =
            calculate_compression_ratio(original_size.as_u64(), compressed_size.as_u64());

        print_success(&format!(
            "Video compressed successfully: {} -> {} ({:.1}% reduction)",
            original_size, compressed_size, compression_ratio
        ));

        Ok(output_path)
    }

    fn get_preset_config(&self, options: &VideoCompressionOptions) -> Result<VideoPresetConfig> {
        if let Some(preset_config) = self.config.get_video_preset(&options.preset) {
            let mut config = preset_config.clone();

            // Override with command-line options
            if let Some(codec) = &options.codec {
                config.codec = codec.clone();
            }
            if let Some(crf) = options.crf {
                config.crf = Some(crf);
            }
            if let Some(bitrate) = &options.bitrate {
                config.bitrate = Some(bitrate.clone());
            }
            if let Some(audio_codec) = &options.audio_codec {
                config.audio_codec = audio_codec.clone();
            }
            if let Some(audio_bitrate) = &options.audio_bitrate {
                config.audio_bitrate = Some(audio_bitrate.clone());
            }
            if options.two_pass {
                config.two_pass = true;
            }

            Ok(config)
        } else {
            Err(CompressError::config(format!(
                "Unknown preset: {}",
                options.preset
            )))
        }
    }

    fn generate_output_path(&self, options: &VideoCompressionOptions) -> Result<PathBuf> {
        if let Some(output) = &options.output {
            Ok(output.clone())
        } else {
            let suffix = format!("_compressed_{}", options.preset);
            let output_path = generate_output_path(
                &options.input,
                options.output_dir.as_deref(),
                Some(&suffix),
                Some("mp4"), // Default to mp4
            );
            Ok(output_path)
        }
    }

    fn build_ffmpeg_command(
        &self,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
        output_path: &Path,
    ) -> Result<Command> {
        let mut cmd = Command::new("ffmpeg");

        // Input file
        cmd.arg("-i").arg(&options.input);

        // Start time
        if let Some(start) = &options.start {
            let start_seconds = parse_time(start)?;
            cmd.arg("-ss").arg(start_seconds.to_string());
        }

        // End time / duration
        if let Some(end) = &options.end {
            if let Some(start) = &options.start {
                let start_seconds = parse_time(start)?;
                let end_seconds = parse_time(end)?;
                let duration = end_seconds - start_seconds;
                cmd.arg("-t").arg(duration.to_string());
            } else {
                let end_seconds = parse_time(end)?;
                cmd.arg("-t").arg(end_seconds.to_string());
            }
        }

        // Video codec
        cmd.arg("-c:v").arg(preset_config.codec.to_string());

        // Video quality/bitrate
        if let Some(bitrate) = &preset_config.bitrate {
            cmd.arg("-b:v").arg(bitrate);
        } else if let Some(crf) = preset_config.crf {
            cmd.arg("-crf").arg(crf.to_string());
        }

        // Preset
        if !preset_config.preset.is_empty() {
            cmd.arg("-preset").arg(&preset_config.preset);
        }

        // Resolution
        if let Some(resolution) = &options.resolution {
            let (width, height) = parse_resolution(resolution)?;
            cmd.arg("-vf").arg(format!("scale={}:{}", width, height));
        }

        // Frame rate
        if let Some(fps) = options.fps {
            cmd.arg("-r").arg(fps.to_string());
        }

        // Audio handling
        if options.no_audio {
            cmd.arg("-an");
        } else {
            cmd.arg("-c:a").arg(preset_config.audio_codec.to_string());
            if let Some(audio_bitrate) = &preset_config.audio_bitrate {
                cmd.arg("-b:a").arg(audio_bitrate);
            }
        }

        // Extra arguments from preset
        for arg in &preset_config.extra_args {
            cmd.arg(arg);
        }

        // Progress and overwrite
        cmd.arg("-progress").arg("pipe:1");
        cmd.arg("-y"); // Overwrite output file

        // Output file
        cmd.arg(output_path);

        // Set up stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        Ok(cmd)
    }

    async fn execute_single_pass_compression(
        &self,
        cmd: &mut Command,
        _duration: f64,
    ) -> Result<()> {
        debug!("Executing FFmpeg command: {:?}", cmd);

        let mut child = cmd
            .spawn()
            .map_err(|e| CompressError::process_failed(format!("Failed to start FFmpeg: {}", e)))?;

        // Handle stdout for progress
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line?;
                if self.verbose {
                    debug!("FFmpeg output: {}", line);
                }
                // TODO: Parse progress and update progress bar
            }
        }

        let status = child.wait().map_err(|e| {
            CompressError::process_failed(format!("Failed to wait for FFmpeg: {}", e))
        })?;

        if !status.success() {
            return Err(CompressError::process_failed("FFmpeg process failed"));
        }

        Ok(())
    }

    async fn execute_two_pass_compression(
        &self,
        _cmd: &mut Command,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
        output_path: &Path,
        duration: f64,
    ) -> Result<()> {
        info!("Starting two-pass compression...");

        // First pass
        let mut first_pass_cmd = self.build_ffmpeg_command(options, preset_config, output_path)?;
        first_pass_cmd.arg("-pass").arg("1");
        first_pass_cmd.arg("-f").arg("null");
        first_pass_cmd.arg("/dev/null"); // Discard output on first pass

        info!("Pass 1/2: Analyzing video...");
        self.execute_single_pass_compression(&mut first_pass_cmd, duration)
            .await?;

        // Second pass
        let mut second_pass_cmd = self.build_ffmpeg_command(options, preset_config, output_path)?;
        second_pass_cmd.arg("-pass").arg("2");

        info!("Pass 2/2: Encoding video...");
        self.execute_single_pass_compression(&mut second_pass_cmd, duration)
            .await?;

        Ok(())
    }

    fn get_video_duration(&self, input: &Path) -> Result<f64> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("csv=p=0")
            .arg(input)
            .output()
            .map_err(|_| CompressError::missing_dependency("ffprobe"))?;

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = duration_str
            .trim()
            .parse()
            .map_err(|_| CompressError::process_failed("Failed to parse video duration"))?;

        Ok(duration)
    }

    fn print_dry_run_info(
        &self,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
        output_path: &Path,
    ) {
        println!(
            "\n{}",
            console::style("DRY RUN - No files will be modified")
                .yellow()
                .bold()
        );
        println!("Input:  {}", options.input.display());
        println!("Output: {}", output_path.display());
        println!("Preset: {}", options.preset);
        println!("Codec:  {}", preset_config.codec);

        if let Some(crf) = preset_config.crf {
            println!("CRF:    {}", crf);
        }
        if let Some(bitrate) = &preset_config.bitrate {
            println!("Bitrate: {}", bitrate);
        }
        if let Some(resolution) = &options.resolution {
            println!("Resolution: {}", resolution);
        }
        if let Some(fps) = options.fps {
            println!("FPS:    {}", fps);
        }

        let audio_info = if options.no_audio {
            "Disabled".to_string()
        } else {
            preset_config.audio_codec.to_string()
        };
        println!("Audio:  {}", audio_info);

        if preset_config.two_pass {
            println!("Mode:   Two-pass encoding");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_output_path() {
        let input = PathBuf::from("/test/input.mp4");
        let options = VideoCompressionOptions {
            input: input.clone(),
            output: None,
            preset: VideoPreset::Medium,
            codec: None,
            crf: None,
            bitrate: None,
            resolution: None,
            fps: None,
            audio_codec: None,
            audio_bitrate: None,
            no_audio: false,
            start: None,
            end: None,
            two_pass: false,
            output_dir: None,
            overwrite: false,
        };

        let config = Config::default();
        let compressor = VideoCompressor::new(config, false, false);
        let output = compressor.generate_output_path(&options).unwrap();

        assert!(output.to_string_lossy().contains("_compressed_medium"));
        assert!(output.extension().unwrap() == "mp4");
    }
}
