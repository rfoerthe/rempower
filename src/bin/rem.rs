//! Rempower binary entry point
//!
//! This is the main executable that parses command-line arguments
//! and dispatches to the appropriate subcommand handler.

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use rempower::cli::{Cli, Commands};
use rempower::subcommands::dns::perform;
use std::error::Error;
use std::io;

/// Main entry point
///
/// Parses CLI arguments and dispatches to the appropriate subcommand.
///
/// # Errors
///
/// Returns an error if any subcommand execution fails.
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dns(args) => {
            perform(args)?;
        }
        Commands::Completions { shell } => {
            generate_completions(shell);
        }

    }

    Ok(())
}

/// Generates shell completions for the specified shell
///
/// Outputs completion script to stdout.
///
/// # Arguments
///
/// * `shell` - The shell to generate completions for (bash, zsh, fish, etc.)
fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "rem", &mut io::stdout());
}
