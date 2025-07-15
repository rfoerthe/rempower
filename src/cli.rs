use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rem",
    about = "Rust empowered tools",
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

impl DnsArgs {
    pub fn is_pub_dns(&self) -> bool {
        self.pub_dns // dhcp is false when pub_dns is true (via ArgGroup)
    }
}
