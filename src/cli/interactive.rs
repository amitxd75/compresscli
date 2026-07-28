//! Interactive wizard mode for step-by-step terminal prompts

use crate::cli::commands::{self, BatchCommandParams, ImageCommandParams, VideoCommandParams};
use crate::core::types::{HwAccelMode, ImageFormat, VideoPreset};
use crate::core::{CompressError, Config, Result};
use crate::ui::progress::{print_header, print_info, print_success};
use crate::utils::{is_image_file, is_video_file};
use console::style;
use std::io::{self, Write};
use std::path::PathBuf;

/// Runs the interactive terminal wizard
pub async fn run_interactive_wizard(
    config: Config,
    dry_run: bool,
    verbose: bool,
    global_gpu: bool,
    global_hwaccel: Option<HwAccelMode>,
) -> Result<()> {
    print_header("CompressCLI Interactive Mode");
    println!("Guided step-by-step compression wizard\n");

    // Step 1: Input Path
    let input_path = loop {
        print!(
            "{} Input file or directory path: ",
            style("?").cyan().bold()
        );
        io::stdout().flush().ok();

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(CompressError::Io)?;
        let trimmed = line.trim().trim_matches('"').trim_matches('\'');

        if trimmed.is_empty() {
            println!("{}", style("Please enter a non-empty path.").yellow());
            continue;
        }

        let path = PathBuf::from(trimmed);
        if !path.exists() {
            println!(
                "{}",
                style(format!(
                    "Path '{}' does not exist. Please try again.",
                    trimmed
                ))
                .yellow()
            );
            continue;
        }

        break path;
    };

    // Step 2: Detect File Type & Dispatch
    if input_path.is_dir() {
        print_info("Detected directory input. Preparing batch compression...");
        run_batch_wizard(input_path, config, dry_run, verbose, global_hwaccel).await
    } else if is_image_file(&input_path) {
        print_info("Detected image input. Preparing image compression...");
        run_image_wizard(input_path, config, dry_run, verbose).await
    } else if is_video_file(&input_path) {
        print_info("Detected video input. Preparing video compression...");
        run_video_wizard(
            input_path,
            config,
            dry_run,
            verbose,
            global_gpu,
            global_hwaccel,
        )
        .await
    } else {
        println!(
            "{}",
            style("Target is neither a standard image, video, nor directory.").yellow()
        );
        print!(
            "{} Treat as video or image? (v/i): ",
            style("?").cyan().bold()
        );
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();
        if choice.trim().eq_ignore_ascii_case("i") {
            run_image_wizard(input_path, config, dry_run, verbose).await
        } else {
            run_video_wizard(
                input_path,
                config,
                dry_run,
                verbose,
                global_gpu,
                global_hwaccel,
            )
            .await
        }
    }
}

async fn run_video_wizard(
    input: PathBuf,
    config: Config,
    dry_run: bool,
    verbose: bool,
    global_gpu: bool,
    global_hwaccel: Option<HwAccelMode>,
) -> Result<()> {
    // Preset Selection
    println!("\nSelect Video Preset:");
    println!("  1. fast      (Faster compression, larger file size)");
    println!("  2. medium    (Balanced compression - default)");
    println!("  3. slow      (Slower compression, smaller file size)");
    println!("  4. ultrafast (Maximum speed)");
    println!("  5. veryslow  (Maximum compression)");
    print!("Choose preset [1-5, default 2]: ");
    io::stdout().flush().ok();

    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    let preset = match line.trim() {
        "1" => VideoPreset::Fast,
        "3" => VideoPreset::Slow,
        "4" => VideoPreset::Ultrafast,
        "5" => VideoPreset::Veryslow,
        _ => VideoPreset::Medium,
    };

    // Format Conversion Option
    print!("Convert video format? (e.g. mp4, mkv, webm, or press Enter for default): ");
    io::stdout().flush().ok();
    let mut fmt_line = String::new();
    io::stdin().read_line(&mut fmt_line).ok();
    let format = fmt_line.trim();

    let output = if !format.is_empty() {
        let ext = format.trim_start_matches('.');
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        let parent = input.parent().unwrap_or_else(|| std::path::Path::new(""));
        Some(parent.join(format!("{}_compressed.{}", stem, ext)))
    } else {
        None
    };

    // GPU Hardware Acceleration Prompt
    let hwaccel = prompt_hwaccel(global_gpu, global_hwaccel);

    // Overwrite prompt
    let overwrite = prompt_confirm("Overwrite output file if it exists? (y/N, default N): ");

    print_success("Configuration complete! Starting video compression...");

    let params = VideoCommandParams {
        input,
        output,
        preset,
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
        hwaccel,
        output_dir: None,
        overwrite,
    };

    commands::handle_video_command(params, config, dry_run, verbose).await
}

async fn run_image_wizard(
    input: PathBuf,
    config: Config,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Quality Prompt
    print!("Enter image quality (1-100, default 85): ");
    io::stdout().flush().ok();
    let mut q_line = String::new();
    io::stdin().read_line(&mut q_line).ok();
    let quality: u8 = q_line.trim().parse().unwrap_or(85).clamp(1, 100);

    // Format Conversion Prompt
    print!("Convert image format? (jpg, png, webp, avif, or press Enter for default): ");
    io::stdout().flush().ok();
    let mut fmt_line = String::new();
    io::stdin().read_line(&mut fmt_line).ok();
    let format = ImageFormat::parse_from_str(fmt_line.trim()).ok();

    // Overwrite prompt
    let overwrite = prompt_confirm("Overwrite output file if it exists? (y/N, default N): ");

    print_success("Configuration complete! Starting image compression...");

    let params = ImageCommandParams {
        input,
        output: None,
        quality,
        format,
        resize: None,
        max_width: None,
        max_height: None,
        optimize: true,
        progressive: false,
        lossless: false,
        preset: None,
        output_dir: None,
        overwrite,
    };

    commands::handle_image_command(params, config, dry_run, verbose).await
}

async fn run_batch_wizard(
    directory: PathBuf,
    config: Config,
    dry_run: bool,
    verbose: bool,
    global_hwaccel: Option<HwAccelMode>,
) -> Result<()> {
    // Process types prompt
    print!("Process videos, images, or both? (v/i/both, default both): ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice_str = choice.trim().to_lowercase();

    let (videos, images) = match choice_str.as_str() {
        "v" | "video" => (true, false),
        "i" | "image" => (false, true),
        _ => (true, true),
    };

    // Parallel jobs prompt
    let default_jobs = num_cpus::get().max(1);
    print!("Max parallel jobs [default {}]: ", default_jobs);
    io::stdout().flush().ok();
    let mut jobs_line = String::new();
    io::stdin().read_line(&mut jobs_line).ok();
    let jobs: usize = jobs_line.trim().parse().unwrap_or(default_jobs).max(1);

    // Overwrite prompt
    let overwrite = prompt_confirm("Overwrite output files if they exist? (y/N, default N): ");

    let _ = global_hwaccel;

    print_success("Configuration complete! Starting batch processing...");

    let params = BatchCommandParams {
        directory,
        pattern: "*".to_string(),
        videos,
        images,
        recursive: false,
        video_preset: VideoPreset::Medium,
        image_quality: 85,
        jobs,
        output_dir: None,
        overwrite,
    };

    commands::handle_batch_command(params, config, dry_run, verbose).await
}

fn prompt_confirm(prompt_msg: &str) -> bool {
    print!("{}", prompt_msg);
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    line.trim().eq_ignore_ascii_case("y")
}

fn prompt_hwaccel(global_gpu: bool, global_hwaccel: Option<HwAccelMode>) -> Option<HwAccelMode> {
    if global_gpu || global_hwaccel.is_some() {
        if global_gpu {
            Some(HwAccelMode::Auto)
        } else {
            global_hwaccel
        }
    } else if prompt_confirm("Enable GPU hardware acceleration? (y/n, default n): ") {
        Some(HwAccelMode::Auto)
    } else {
        None
    }
}
