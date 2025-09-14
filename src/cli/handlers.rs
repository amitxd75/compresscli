//! Command handlers and main CLI execution logic
//!
//! This module contains the main CLI execution logic and command routing,
//! including preset management and configuration loading.

use crate::cli::args::{Cli, Commands, PresetAction};
use crate::cli::commands::{self, BatchCommandParams, ImageCommandParams, VideoCommandParams};
use crate::core::{CompressError, Config, ImagePresetConfig, Result, VideoPresetConfig};
use crate::ui::progress::{print_header, print_success};

/// Main CLI execution function
/// Loads configuration and dispatches to appropriate command handlers
pub async fn run_cli(cli: Cli) -> Result<()> {
    // Load configuration from file or create default
    let config = load_config(&cli)?;

    match cli.command {
        Commands::Video {
            input,
            output,
            preset,
            codec,
            crf,
            bitrate,
            resolution,
            fps,
            audio_codec,
            audio_bitrate,
            no_audio,
            start,
            end,
            two_pass,
        } => {
            let params = VideoCommandParams {
                input,
                output,
                preset,
                codec,
                crf,
                bitrate,
                resolution,
                fps,
                audio_codec,
                audio_bitrate,
                no_audio,
                start,
                end,
                two_pass,
                output_dir: cli.output_dir,
                overwrite: cli.overwrite,
            };
            commands::handle_video_command(params, config, cli.dry_run, cli.verbose).await?;
        }

        Commands::Image {
            input,
            output,
            quality,
            format,
            resize,
            max_width,
            max_height,
            optimize,
            progressive,
            lossless,
            preset,
        } => {
            let params = ImageCommandParams {
                input,
                output,
                quality,
                format,
                resize,
                max_width,
                max_height,
                optimize,
                progressive,
                lossless,
                preset,
                output_dir: cli.output_dir,
                overwrite: cli.overwrite,
            };
            commands::handle_image_command(params, config, cli.dry_run, cli.verbose).await?;
        }

        Commands::Batch {
            directory,
            pattern,
            videos,
            images,
            recursive,
            video_preset,
            image_quality,
            jobs,
        } => {
            let params = BatchCommandParams {
                directory,
                pattern,
                videos,
                images,
                recursive,
                video_preset,
                image_quality,
                jobs,
                output_dir: cli.output_dir,
                overwrite: cli.overwrite,
            };
            commands::handle_batch_command(params, config, cli.dry_run, cli.verbose).await?;
        }

        Commands::Presets { action } => {
            handle_presets_command(action, config).await?;
        }

        Commands::Info => {
            commands::handle_info_command().await?;
        }

        Commands::Completions { shell } => {
            commands::handle_completions_command(shell)?;
        }
    }

    Ok(())
}

/// Handles all preset-related commands (list, show, create, delete)
/// Manages user-defined and built-in compression presets
async fn handle_presets_command(action: PresetAction, config: Config) -> Result<()> {
    match action {
        PresetAction::List => {
            print_header("Available Presets");

            println!("\\n{}", console::style("Video Presets:").bold());
            for (name, preset) in &config.video_presets {
                println!(
                    "  {} - {} (CRF: {:?}, Codec: {})",
                    console::style(name).cyan(),
                    preset.preset,
                    preset.crf,
                    preset.codec
                );
            }

            println!("\\n{}", console::style("Image Presets:").bold());
            for (name, preset) in &config.image_presets {
                println!(
                    "  {} - Quality: {}, Optimize: {}",
                    console::style(name).cyan(),
                    preset.quality,
                    preset.optimize
                );
            }
        }

        PresetAction::Show { name } => {
            if let Some(video_preset) = config.video_presets.get(&name) {
                print_header(&format!("Video Preset: {}", name));
                println!("Codec: {}", video_preset.codec);
                println!("CRF: {:?}", video_preset.crf);
                println!("Bitrate: {:?}", video_preset.bitrate);
                println!("Audio Codec: {}", video_preset.audio_codec);
                println!("Audio Bitrate: {:?}", video_preset.audio_bitrate);
                println!("Preset: {}", video_preset.preset);
                println!("Two-pass: {}", video_preset.two_pass);
                if !video_preset.extra_args.is_empty() {
                    println!("Extra args: {:?}", video_preset.extra_args);
                }
            } else if let Some(image_preset) = config.image_presets.get(&name) {
                print_header(&format!("Image Preset: {}", name));
                println!("Quality: {}", image_preset.quality);
                println!("Optimize: {}", image_preset.optimize);
                println!("Progressive: {}", image_preset.progressive);
                println!("Lossless: {}", image_preset.lossless);
            } else {
                return Err(CompressError::config(format!(
                    "Preset '{}' not found",
                    name
                )));
            }
        }

        PresetAction::Create {
            name,
            config: config_file,
        } => {
            // Load preset from file
            let preset_content = std::fs::read_to_string(&config_file)
                .map_err(|e| CompressError::config(format!("Failed to read config file: {}", e)))?;

            // Try to parse as video preset first, then image preset
            if let Ok(video_preset) = serde_yaml::from_str::<VideoPresetConfig>(&preset_content) {
                let mut config = config;
                config.add_video_preset(name.clone(), video_preset);
                config.save_to_file(Config::get_default_config_path()?)?;
                print_success(&format!("Video preset '{}' created successfully", name));
            } else if let Ok(image_preset) =
                serde_yaml::from_str::<ImagePresetConfig>(&preset_content)
            {
                let mut config = config;
                config.add_image_preset(name.clone(), image_preset);
                config.save_to_file(Config::get_default_config_path()?)?;
                print_success(&format!("Image preset '{}' created successfully", name));
            } else {
                return Err(CompressError::config(
                    "Invalid preset format. Must be a valid video or image preset",
                ));
            }
        }

        PresetAction::Delete { name } => {
            let mut config = config;
            let mut deleted = false;

            if config.remove_video_preset(&name) {
                deleted = true;
                print_success(&format!("Video preset '{}' deleted", name));
            }

            if config.remove_image_preset(&name) {
                deleted = true;
                print_success(&format!("Image preset '{}' deleted", name));
            }

            if deleted {
                config.save_to_file(Config::get_default_config_path()?)?;
            } else {
                return Err(CompressError::config(format!(
                    "Preset '{}' not found",
                    name
                )));
            }
        }
    }

    Ok(())
}

/// Loads configuration from file or creates default configuration
/// Uses custom config path if provided, otherwise uses default location
fn load_config(cli: &Cli) -> Result<Config> {
    if let Some(config_path) = &cli.config {
        Config::load_from_file(config_path)
    } else {
        Config::load_or_create_default()
    }
}
