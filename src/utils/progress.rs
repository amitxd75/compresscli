//! Progress tracking utilities for compression operations

use crate::core::{
    CompressError, FFMPEG_PROGRESS_FRAME_PATTERN, FFMPEG_PROGRESS_TIME_PATTERN,
    PROGRESS_UPDATE_INTERVAL_MS, Result,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Manages progress tracking for compression operations
pub struct ProgressManager {
    progress_bar: ProgressBar,
    total_duration: Option<f64>,
}

impl ProgressManager {
    /// Creates a new progress manager for file operations
    pub fn new_file_progress(total_files: usize) -> Self {
        Self {
            progress_bar: crate::ui::progress::create_file_progress_bar(total_files),
            total_duration: None,
        }
    }

    /// Creates a new progress manager for compression operations
    pub fn new_compression_progress(duration: Option<f64>) -> Self {
        let valid_duration =
            duration.filter(|d| d.is_finite() && *d > 0.0 && *d < (86400.0 * 365.0));

        let pb = if let Some(duration) = valid_duration {
            let duration_ms = (duration * 1000.0).min(u64::MAX as f64) as u64;
            let pb = ProgressBar::new(duration_ms); // Convert to milliseconds
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.yellow} [{elapsed_precise}] [{wide_bar:.yellow/blue}] {percent}% {msg}")
                    .unwrap()
                    .progress_chars("ᗧ• "),
            );
            pb.enable_steady_tick(Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MS));
            pb
        } else {
            crate::ui::progress::create_compression_progress_bar()
        };

        Self {
            progress_bar: pb,
            total_duration: valid_duration,
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
            let time_str = time_str.trim();

            // Skip parsing if FFmpeg outputs "N/A" (common at start of encoding)
            if time_str == "N/A" {
                return Ok(());
            }

            let time_microseconds: f64 = time_str.parse().map_err(|_| {
                CompressError::progress_error(format!(
                    "Invalid time format in FFmpeg output: '{}'",
                    time_str
                ))
            })?;

            // Convert microseconds to milliseconds
            let time_ms = time_microseconds / 1000.0;
            self.progress_manager.update_from_time(time_ms);
        } else if let Some(_frame_str) = line.strip_prefix(FFMPEG_PROGRESS_FRAME_PATTERN) {
            // Frame pattern matched; ignored when time-based progress is active
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
}

use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tokio::process::Child as TokioChild;

/// Monitors FFmpeg process output asynchronously and updates progress
pub async fn monitor_ffmpeg_progress(
    mut child: TokioChild,
    parser: FFmpegProgressParser,
) -> Result<()> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stderr_handle = tokio::spawn(async move {
        let mut err_buf = String::new();
        const MAX_STDERR_BYTES: usize = 64 * 1024;
        if let Some(stderr_stream) = stderr {
            let mut reader = TokioBufReader::new(stderr_stream);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                if err_buf.len() < MAX_STDERR_BYTES {
                    err_buf.push_str(&line);
                    if err_buf.len() >= MAX_STDERR_BYTES {
                        err_buf.truncate(MAX_STDERR_BYTES);
                        err_buf.push_str("\n... [stderr truncated]");
                    }
                }
                line.clear();
            }
        }
        err_buf
    });

    if let Some(stdout_stream) = stdout {
        let mut reader = TokioBufReader::new(stdout_stream);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                break;
            }
            let trimmed = line.trim_end();
            if let Err(e) = parser.parse_line(trimmed) {
                log::debug!("Error parsing progress line: {}", e);
            }
            line.clear();
        }
    }

    let status = child.wait().await.map_err(|e| {
        CompressError::ffmpeg_error(format!("Failed to wait for FFmpeg process: {}", e), None)
    })?;

    let stderr_output = stderr_handle.await.unwrap_or_default();

    if !status.success() {
        let err_msg = if stderr_output.trim().is_empty() {
            format!("FFmpeg process exited with status {}", status)
        } else {
            format!("FFmpeg failed: {}", stderr_output.trim())
        };
        return Err(CompressError::ffmpeg_error(err_msg, None));
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

        // Test N/A time parsing (should not error)
        assert!(parser.parse_line("out_time_ms=N/A").is_ok());

        // Test invalid time parsing
        assert!(parser.parse_line("out_time_ms=invalid").is_err());

        // Test non-time line (should not error)
        assert!(parser.parse_line("frame=100").is_ok());

        let parser2 = FFmpegProgressParser::new(Some(50.0));
        parser2.finish();
    }

    #[test]
    fn test_progress_manager_creation() {
        let file_progress = ProgressManager::new_file_progress(10);
        file_progress.inc(1);
        file_progress.finish_and_clear();

        let _compression_progress = ProgressManager::new_compression_progress(Some(120.0));
        let _spinner_progress = ProgressManager::new_compression_progress(None);
    }
}
