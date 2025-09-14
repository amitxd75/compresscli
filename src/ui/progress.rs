use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates a progress bar for tracking file processing in batch operations
/// Shows current progress, elapsed time, and files processed count
pub fn create_file_progress_bar(file_count: usize) -> ProgressBar {
    let pb = ProgressBar::new(file_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} files processed")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Creates a spinner progress bar for compression operations
/// Used when progress percentage is unknown (like FFmpeg processing)
pub fn create_compression_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Prints a success message with a green checkmark
/// Used to indicate successful completion of operations
pub fn print_success(message: &str) {
    println!("{} {}", style("✓").green().bold(), message);
}

/// Prints an error message with a red X mark to stderr
/// Used for error reporting and failure notifications
pub fn print_error(message: &str) {
    eprintln!("{} {}", style("✗").red().bold(), message);
}

/// Prints an informational message with a blue info icon
/// Used for general status updates and information
pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ").blue().bold(), message);
}

/// Prints a formatted header with underline
/// Used for section titles and major operation headers
pub fn print_header(message: &str) {
    println!(
        "\n{}\n{}",
        style(message).bold().underlined(),
        style("─".repeat(message.len())).dim()
    );
}

/// Prints a horizontal separator line
/// Used to visually separate different sections of output
pub fn print_separator() {
    println!("{}", style("─".repeat(50)).dim());
}
