use clap::{Parser, Subcommand};
use clap_complete::Shell;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_about() -> String {
    format!("Rust empowered tools v{}", VERSION)
}

#[derive(Parser)]
#[command(
    name = "rem",
    about = get_about(),
    long_about = None,
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

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
