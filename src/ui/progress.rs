use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates a refined progress bar for tracking file processing in batch operations
/// Uses Pacman progress animation: Pacman 'C' eats dots '•' as progress advances
pub fn create_file_progress_bar(file_count: usize) -> ProgressBar {
    let pb = ProgressBar::new(file_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow} [{elapsed_precise}] [{wide_bar:.yellow/blue}] {pos}/{len} files ({eta})")
            .unwrap()
            .progress_chars("ᗧ• "),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Creates a smooth spinner progress bar for compression operations
/// Used when progress percentage is unknown (e.g. streaming process outputs)
pub fn create_compression_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("ᗧ•  • ᗤ  ")
            .template("{spinner:.yellow} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Prints a success message with a green checkmark
pub fn print_success(message: &str) {
    println!("{} {}", style("✔").green().bold(), message);
}

/// Prints an error message with a red X mark to stderr
pub fn print_error(message: &str) {
    eprintln!("{} {}", style("✖").red().bold(), message);
}

/// Prints a warning message with a yellow warning icon
pub fn print_warning(message: &str) {
    println!("{} {}", style("⚠").yellow().bold(), message);
}

/// Prints an informational message with a cyan info icon
pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ").cyan().bold(), message);
}

/// Prints a formatted section header with horizontal rule styling
pub fn print_header(message: &str) {
    let line_len = 50usize.saturating_sub(message.len() + 4).max(5);
    println!(
        "\n{} {} {}",
        style("───").cyan().dim(),
        style(message).bold().cyan(),
        style("─".repeat(line_len)).cyan().dim()
    );
}

/// Prints a subtle horizontal separator line
pub fn print_separator() {
    println!(
        "{}",
        style("──────────────────────────────────────────────────").dim()
    );
}

/// Prints a styled status badge (e.g. [CACHED], [GPU], [BATCH])
pub fn print_badge(label: &str, message: &str) {
    println!(
        "{} {}",
        style(format!("[{}]", label)).bold().magenta(),
        message
    );
}
