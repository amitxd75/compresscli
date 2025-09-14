//! Command-line interface module for CompressCLI
//!
//! This module contains all CLI-related functionality including command definitions,
//! argument parsing, and command handlers.

pub mod args;
pub mod commands;
pub mod handlers;

// Re-export main CLI types
pub use args::Cli;
pub use handlers::run_cli;
