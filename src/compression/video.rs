use crate::core::types::{VideoCompressionOptions, VideoPreset};
use crate::core::{CompressError, Config, DEFAULT_VIDEO_EXTENSION, Result, VideoPresetConfig};
use crate::ui::progress::print_success;
use crate::utils::{
    FFmpegCommandBuilder, FFmpegProgressParser, FFprobeCommandBuilder, check_output_overwrite,
    ensure_parent_dir, generate_output_path, get_file_size, monitor_ffmpeg_progress,
    validate_input_file, validate_safe_path,
};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

pub struct VideoCompressor {
    pub config: Config,
    pub dry_run: bool,
    pub verbose: bool,
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
        // Enforce invariants using configured max_fps
        options.validate_with_max_fps(self.config.default_settings.max_fps)?;
        if let Some(crf) = options.crf {
            debug_assert!(crf <= 51, "CRF invariant violated");
        }

        // Validate input file exists and is accessible
        validate_input_file(&options.input)?;
        validate_safe_path(&options.input)?;

        // Get video preset configuration from config
        let preset_config = self.get_preset_config(&options)?;

        // Generate output path with appropriate naming
        let output_path = self.generate_output_path(&options)?;

        // Ensure parent directory exists
        ensure_parent_dir(&output_path)?;

        // Check if we should overwrite existing files
        check_output_overwrite(&output_path, options.overwrite)?;

        // Get original file size
        let original_size = get_file_size(&options.input)?;

        info!(
            "Compressing video: {} -> {}",
            options.input.display(),
            output_path.display()
        );

        // Check cache signature using deterministic hash
        let options_hash = options.options_hash();
        if crate::core::CacheManager::check_cache_hit(&options.input, &output_path, &options_hash) {
            return Ok(output_path);
        }

        if self.dry_run {
            self.print_dry_run_info(&options, &preset_config, &output_path);
            return Ok(output_path);
        }

        // Get video duration for progress tracking
        let duration = self.get_video_duration(&options.input).await?;

        // Execute compression with graceful CPU fallback if hardware encoder fails
        let result = if preset_config.two_pass {
            if preset_config.bitrate.is_none() {
                crate::ui::progress::print_warning(
                    "Two-pass encoding requested without a target bitrate. Falling back to single-pass CRF compression.",
                );
                warn!(
                    "Two-pass encoding requested without a target bitrate. Falling back to single-pass CRF compression."
                );
                self.execute_single_pass_compression(
                    &options,
                    &preset_config,
                    &output_path,
                    duration,
                )
                .await
            } else {
                self.execute_two_pass_compression(&options, &preset_config, &output_path, duration)
                    .await
            }
        } else {
            self.execute_single_pass_compression(&options, &preset_config, &output_path, duration)
                .await
        };

        if let Err(e) = result {
            if options.hwaccel.is_some() {
                warn!(
                    "Hardware acceleration failed: {}. Falling back to CPU software encoding...",
                    e
                );
                let mut fallback_options = options.clone();
                fallback_options.hwaccel = None;

                if preset_config.two_pass && preset_config.bitrate.is_some() {
                    self.execute_two_pass_compression(
                        &fallback_options,
                        &preset_config,
                        &output_path,
                        duration,
                    )
                    .await?;
                } else {
                    self.execute_single_pass_compression(
                        &fallback_options,
                        &preset_config,
                        &output_path,
                        duration,
                    )
                    .await?;
                }
            } else {
                return Err(e);
            }
        }

        // Get compressed file size and calculate ratio
        let compressed_size = get_file_size(&output_path)?;
        let ratio_str = crate::utils::math::format_compression_ratio(
            original_size.as_u64(),
            compressed_size.as_u64(),
        );

        // Record in cache
        crate::core::CacheManager::record_cache(&options.input, &output_path, &options_hash);

        print_success(&format!(
            "Video compressed successfully: {} -> {} ({})",
            original_size, compressed_size, ratio_str
        ));

        Ok(output_path)
    }

    /// Gets preset configuration with command-line overrides applied
    fn get_preset_config(&self, options: &VideoCompressionOptions) -> Result<VideoPresetConfig> {
        if matches!(options.preset, VideoPreset::Custom) {
            if let Some(preset_config) = self.config.get_video_preset(&options.preset) {
                let mut config = preset_config.clone();
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
                return Ok(config);
            }
            return Err(CompressError::config(
                "Preset 'custom' is not defined. Create custom preset first using 'compresscli presets create custom <config_file>' or specify individual flags.",
            ));
        }

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

    /// Generates output path with proper naming and validation
    fn generate_output_path(&self, options: &VideoCompressionOptions) -> Result<PathBuf> {
        if let Some(output) = &options.output {
            validate_safe_path(output)?;
            Ok(output.clone())
        } else {
            let suffix = format!("_compressed_{}", options.preset);
            let output_path = generate_output_path(
                &options.input,
                options.output_dir.as_deref(),
                Some(&suffix),
                Some(DEFAULT_VIDEO_EXTENSION),
            );
            Ok(output_path)
        }
    }

    /// Builds base FFmpeg command options without attaching final output file argument
    fn build_ffmpeg_base(
        &self,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
    ) -> Result<FFmpegCommandBuilder> {
        let mut builder = FFmpegCommandBuilder::new().input(&options.input)?;

        if let Some(hwaccel_mode) = &options.hwaccel {
            builder = builder.hardware_video_codec(&preset_config.codec, hwaccel_mode);
        } else {
            builder = builder.video_codec(preset_config.codec.clone());
        }

        builder = builder
            .preset_for_codec(&preset_config.preset, &preset_config.codec)
            .progress()
            .overwrite();

        if let Some(threads) = options.threads {
            builder = builder.threads(threads);
        }

        // Video quality/bitrate
        if let Some(bitrate) = &preset_config.bitrate {
            builder = builder.bitrate(bitrate)?;
        } else if let Some(crf) = preset_config.crf {
            builder = builder.crf(crf)?;
        }

        // Start time
        if let Some(start) = &options.start {
            builder = builder.start_time(start)?;
        }

        // Duration (calculated from start and end times)
        if let Some(end) = &options.end {
            if let Some(start) = &options.start {
                let start_seconds = crate::utils::parse_time(start)?;
                let end_seconds = crate::utils::parse_time(end)?;
                if end_seconds <= start_seconds {
                    return Err(CompressError::invalid_parameter(
                        "end",
                        "End time must be strictly greater than start time",
                    ));
                }
                let duration = end_seconds - start_seconds;
                builder = builder.duration(&duration.to_string())?;
            } else {
                builder = builder.duration(end)?;
            }
        }

        // Resolution
        if let Some(resolution) = &options.resolution {
            builder = builder.resolution(resolution)?;
        }

        // Frame rate
        if let Some(fps) = options.fps {
            builder = builder.framerate(fps)?;
        }

        // Audio handling
        if options.no_audio {
            builder = builder.no_audio();
        } else {
            builder = builder.audio_codec(preset_config.audio_codec.clone());
            if let Some(audio_bitrate) = &preset_config.audio_bitrate {
                builder = builder.audio_bitrate(audio_bitrate)?;
            }
        }

        // Extra arguments from preset
        if !preset_config.extra_args.is_empty() {
            builder = builder.custom_args(&preset_config.extra_args)?;
        }

        Ok(builder)
    }

    /// Builds FFmpeg command with output file specified
    fn build_ffmpeg_command(
        &self,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
        output_path: &Path,
    ) -> Result<FFmpegCommandBuilder> {
        let builder = self.build_ffmpeg_base(options, preset_config)?;
        builder.output(output_path)
    }

    /// Executes single-pass compression with progress tracking
    async fn execute_single_pass_compression(
        &self,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
        output_path: &Path,
        duration: Option<f64>,
    ) -> Result<()> {
        let builder = self.build_ffmpeg_command(options, preset_config, output_path)?;
        let mut command = builder.build_tokio();

        if self.verbose {
            debug!("Executing FFmpeg command: {:?}", command);
        }

        let child = command.spawn().map_err(|e| {
            CompressError::ffmpeg_error(
                format!("Failed to start FFmpeg: {}", e),
                Some(format!("{:?}", command)),
            )
        })?;

        let progress_parser = FFmpegProgressParser::new(duration);
        progress_parser.set_message("Compressing video...");

        monitor_ffmpeg_progress(child, progress_parser).await?;

        Ok(())
    }

    /// Executes two-pass compression with progress tracking and automatic log cleanup
    async fn execute_two_pass_compression(
        &self,
        options: &VideoCompressionOptions,
        preset_config: &VideoPresetConfig,
        output_path: &Path,
        duration: Option<f64>,
    ) -> Result<()> {
        info!("Starting two-pass compression...");

        let cleanup_pass_logs = || {
            let _ = std::fs::remove_file("ffmpeg2pass-0.log");
            let _ = std::fs::remove_file("ffmpeg2pass-0.log.mbtree");
        };

        // First pass - outputs to NULL_DEVICE without attaching final output file path
        let first_pass_builder = self.build_ffmpeg_base(options, preset_config)?.first_pass();
        let mut first_pass_cmd = first_pass_builder.build_tokio();

        if self.verbose {
            debug!("First pass command: {:?}", first_pass_cmd);
        }

        let first_pass_child = first_pass_cmd.spawn().map_err(|e| {
            cleanup_pass_logs();
            CompressError::ffmpeg_error(
                format!("Failed to start first pass: {}", e),
                Some(format!("{:?}", first_pass_cmd)),
            )
        })?;

        let first_pass_parser = FFmpegProgressParser::new(duration);
        first_pass_parser.set_message("Pass 1/2: Analyzing video...");

        if let Err(e) = monitor_ffmpeg_progress(first_pass_child, first_pass_parser).await {
            cleanup_pass_logs();
            return Err(e);
        }

        // Second pass - outputs to output_path
        let second_pass_builder = self
            .build_ffmpeg_command(options, preset_config, output_path)?
            .second_pass();
        let mut second_pass_cmd = second_pass_builder.build_tokio();

        if self.verbose {
            debug!("Second pass command: {:?}", second_pass_cmd);
        }

        let second_pass_child = second_pass_cmd.spawn().map_err(|e| {
            cleanup_pass_logs();
            CompressError::ffmpeg_error(
                format!("Failed to start second pass: {}", e),
                Some(format!("{:?}", second_pass_cmd)),
            )
        })?;

        let second_pass_parser = FFmpegProgressParser::new(duration);
        second_pass_parser.set_message("Pass 2/2: Encoding video...");

        let res = monitor_ffmpeg_progress(second_pass_child, second_pass_parser).await;
        cleanup_pass_logs();
        res
    }

    /// Gets video duration using FFprobe
    async fn get_video_duration(&self, input: &Path) -> Result<Option<f64>> {
        let mut command = FFprobeCommandBuilder::new()
            .input(input)?
            .duration()
            .build();

        let output = command.output().map_err(|e| {
            CompressError::ffmpeg_error(
                format!("Failed to run FFprobe: {}", e),
                Some(format!("{:?}", command)),
            )
        })?;

        if !output.status.success() {
            warn!("FFprobe failed to get duration, continuing without progress tracking");
            return Ok(None);
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = duration_str.trim().parse().map_err(|e| {
            CompressError::progress_error(format!("Failed to parse video duration: {}", e))
        })?;

        Ok(Some(duration))
    }

    /// Prints dry run information
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

// Make VideoCompressor cloneable for async processing
impl Clone for VideoCompressor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            dry_run: self.dry_run,
            verbose: self.verbose,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::VideoCodec;

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
            threads: None,
            hwaccel: None,
            output_dir: None,
            overwrite: false,
        };

        let config = Config::default();
        let compressor = VideoCompressor::new(config, false, false);
        let output = compressor.generate_output_path(&options).unwrap();

        assert!(output.to_string_lossy().contains("_compressed_medium"));
        assert!(output.extension().unwrap() == "mp4");
    }

    #[test]
    fn test_preset_config_override() {
        let config = Config::default();
        let compressor = VideoCompressor::new(config, false, false);

        let options = VideoCompressionOptions {
            input: PathBuf::from("test.mp4"),
            output: None,
            preset: VideoPreset::Medium,
            codec: Some(VideoCodec::H265),
            crf: Some(20),
            bitrate: None,
            resolution: None,
            fps: None,
            audio_codec: None,
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

        let preset_config = compressor.get_preset_config(&options).unwrap();
        assert!(matches!(preset_config.codec, VideoCodec::H265));
        assert_eq!(preset_config.crf, Some(20));
    }
}
