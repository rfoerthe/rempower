use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rem",
    about = "Rust empowered tools",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage DNS server settings
    Dns(DnsArgs),
}

#[derive(Parser)]
#[group(required = true, multiple = false)]
pub struct DnsArgs {
    /// Enable CloudFlare and Google DNS servers
    #[arg(long)]
    pub enable: bool,

    /// Reset to DNS servers configured in router (disable custom DNS)
    #[arg(long)]
    pub reset: bool,
}

impl DnsArgs {
    pub fn is_pub_dns(&self) -> bool {
        self.enable // reset ist false wenn enable true ist (durch ArgGroup)
    }
}

