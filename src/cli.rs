//! Command-line interface definitions
//!
//! This module defines the CLI structure using clap's derive macros.
//! It includes the root command parser and all subcommand definitions.

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Package version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Generates the about text with version information
fn get_about() -> String {
    format!("Rust empowered tools v{}", VERSION)
}

/// Root CLI structure
///
/// Parses the command line and dispatches to the appropriate subcommand.
#[derive(Parser)]
#[command(
    name = "rem",
    about = get_about(),
    long_about = None,
    version
)]
pub struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands
///
/// Each variant represents a different tool or utility provided by rempower.
#[derive(Subcommand)]
pub enum Commands {
    /// Switch between public DNS servers and those assigned by the DHCP server
    Dns(DnsArgs),
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Arguments for the DNS subcommand
///
/// Provides options to configure DNS settings on macOS network interfaces.
/// Only one option can be specified at a time.
#[derive(Parser)]
#[group(required = true, multiple = false)]
pub struct DnsArgs {
    /// Enable CloudFlare and Google DNS servers
    #[arg(long = "pub")]
    pub pub_dns: bool,

    /// Revert to DNS servers assigned by the DHCP server
    #[arg(long = "dhcp")]
    pub dhcp: bool,

    /// List active DNS servers
    #[arg(short,long)]
    pub list: bool,
}
