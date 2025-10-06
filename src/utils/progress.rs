//! Progress tracking utilities for compression operations

use crate::core::{
    CompressError, FFMPEG_PROGRESS_TIME_PATTERN, PROGRESS_UPDATE_INTERVAL_MS, Result,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader};
use std::process::Child;
use std::time::Duration;

/// Manages progress tracking for compression operations
pub struct ProgressManager {
    progress_bar: ProgressBar,
    total_duration: Option<f64>,
}

impl ProgressManager {
    /// Creates a new progress manager for file operations
    pub fn new_file_progress(total_files: usize) -> Self {
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} files processed")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.enable_steady_tick(Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MS));

        Self {
            progress_bar: pb,
            total_duration: None,
        }
    }

    /// Creates a new progress manager for compression operations
    pub fn new_compression_progress(duration: Option<f64>) -> Self {
        let pb = if let Some(duration) = duration {
            let pb = ProgressBar::new((duration * 1000.0) as u64); // Convert to milliseconds
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb
        };

        pb.enable_steady_tick(Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MS));

        Self {
            progress_bar: pb,
            total_duration: duration,
        }
    }

    /// Sets the progress message
    pub fn set_message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    /// Increments progress by one unit
    pub fn inc(&self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    /// Sets the current progress position
    #[allow(dead_code)]
    pub fn set_position(&self, pos: u64) {
        self.progress_bar.set_position(pos);
    }

    /// Updates progress based on FFmpeg time output
    pub fn update_from_time(&self, time_ms: f64) {
        if let Some(total) = self.total_duration {
            let progress = (time_ms / 1000.0 / total * 100.0).min(100.0);
            self.progress_bar
                .set_position((time_ms / 1000.0 * 1000.0) as u64);
            self.set_message(&format!("Compressing... {:.1}%", progress));
        }
    }

    /// Finishes the progress bar and clears it
    pub fn finish_and_clear(self) {
        self.progress_bar.finish_and_clear();
    }

    /// Finishes the progress bar with a message
    #[allow(dead_code)]
    pub fn finish_with_message(self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }
}

/// Parses FFmpeg progress output and updates progress bar
pub struct FFmpegProgressParser {
    progress_manager: ProgressManager,
}

impl FFmpegProgressParser {
    /// Creates a new FFmpeg progress parser
    pub fn new(duration: Option<f64>) -> Self {
        Self {
            progress_manager: ProgressManager::new_compression_progress(duration),
        }
    }

    /// Parses a line of FFmpeg output and updates progress
    pub fn parse_line(&self, line: &str) -> Result<()> {
        if let Some(time_str) = line.strip_prefix(FFMPEG_PROGRESS_TIME_PATTERN) {
            let time_microseconds: f64 = time_str.trim().parse().map_err(|_| {
                CompressError::progress_error("Invalid time format in FFmpeg output")
            })?;

            // Convert microseconds to milliseconds
            let time_ms = time_microseconds / 1000.0;
            self.progress_manager.update_from_time(time_ms);
        }
        Ok(())
    }

    /// Sets a message on the progress bar
    pub fn set_message(&self, message: &str) {
        self.progress_manager.set_message(message);
    }

    /// Finishes the progress tracking
    pub fn finish(self) {
        self.progress_manager.finish_and_clear();
    }

    /// Finishes with a specific message
    #[allow(dead_code)]
    pub fn finish_with_message(self, message: &str) {
        self.progress_manager.finish_with_message(message);
    }
}

/// Monitors FFmpeg process output and updates progress
pub async fn monitor_ffmpeg_progress(mut child: Child, parser: FFmpegProgressParser) -> Result<()> {
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line.map_err(CompressError::Io)?;
            parser.parse_line(&line)?;
        }
    }

    let status = child.wait().map_err(|e| {
        CompressError::ffmpeg_error(format!("Failed to wait for FFmpeg process: {}", e), None)
    })?;

    if !status.success() {
        return Err(CompressError::ffmpeg_error("FFmpeg process failed", None));
    }

    parser.finish();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_parser() {
        let parser = FFmpegProgressParser::new(Some(100.0));

        // Test valid time parsing
        assert!(parser.parse_line("out_time_ms=50000000").is_ok()); // 50 seconds in microseconds

        // Test invalid time parsing
        assert!(parser.parse_line("out_time_ms=invalid").is_err());

        // Test non-time line (should not error)
        assert!(parser.parse_line("frame=100").is_ok());
    }

    #[test]
    fn test_progress_manager_creation() {
        let _file_progress = ProgressManager::new_file_progress(10);
        let _compression_progress = ProgressManager::new_compression_progress(Some(120.0));
        let _spinner_progress = ProgressManager::new_compression_progress(None);
    }
}
