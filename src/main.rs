//! CompressCLI - A powerful CLI tool for video and image compression
//!
//! This is the main entry point for the CompressCLI application.
//! It initializes logging, parses CLI arguments, and delegates to the CLI handler.

mod cli;
mod compression;
mod core;
mod ui;
mod utils;

use cli::{run_cli, Cli};
use clap::Parser;
use std::process;
use ui::progress::print_error;

/// Main entry point for the CompressCLI application
/// Initializes logging, parses CLI arguments, and runs the main logic
#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = run_cli(cli).await {
        print_error(&format!("Error: {}", e));
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Config;

    #[tokio::test]
    async fn test_config_loading() {
        let config = Config::default();
        assert!(!config.video_presets.is_empty());
        assert!(!config.image_presets.is_empty());
    }
}