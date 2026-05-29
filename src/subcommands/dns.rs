//! DNS configuration subcommand
//!
//! Provides functionality to configure DNS servers on macOS network interfaces.
//! Supports switching between public DNS servers (CloudFlare and Google) and
//! DHCP-assigned DNS servers.
//!
//! # System Requirements
//!
//! - macOS operating system
//! - sudo privileges for modifying DNS settings
//!
//! # System Commands Used
//!
//! - `networksetup -listallnetworkservices` - List network interfaces
//! - `networksetup -setdnsservers` - Configure DNS (requires sudo)
//! - `networksetup -getdnsservers` - Get manually configured DNS
//! - `scutil --dns` - Get all DNS configuration including DHCP

use crate::cli::DnsArgs;
use colored::Colorize;
use std::collections::HashSet;
use std::error::Error;
use std::process::Command;

/// Public DNS servers (CloudFlare and Google with IPv4 and IPv6)
const PUBLIC_DNS: &[&str] = &["1.1.1.1", "2606:4700:4700::1111", "8.8.4.4", "2001:4860:4860::8844"];

#[derive(Debug, Clone, PartialEq, Eq)]
enum ManualDns {
    Servers(Vec<String>),
    Dhcp,
}

/// Performs DNS configuration operations based on the provided arguments.
///
/// This function handles three operations:
/// - Switching to public DNS servers (CloudFlare and Google)
/// - Reverting to DHCP-assigned DNS servers
/// - Listing currently configured DNS servers
///
/// # Arguments
///
/// * `args` - DNS operation arguments from the command line
///
/// # Errors
///
/// Returns an error if the DNS configuration update fails or if network commands fail.
pub fn perform(args: DnsArgs) -> Result<(), Box<dyn Error>> {
    if args.dhcp {
        enable_dhcp_dns()?;
    } else if args.pub_dns {
        enable_pub_dns()?;
    } else if args.list {
        print_current_dns()?;
    }

    Ok(())
}

/// Prints current DNS servers for all active network interfaces
///
/// # Errors
///
/// Returns an error if network commands fail or output cannot be parsed.
fn print_current_dns() -> Result<(), Box<dyn Error>> {
    let networks = active_networks()?;

    for network in networks {
        let dns_servers = current_dns_servers(&network)?;
        println!("{network:>30} : {dns_servers:?}");
    }

    Ok(())
}

/// Returns names of active network interfaces
///
/// Uses `networksetup -listallnetworkservices` to get network interfaces,
/// filtering out disabled interfaces (marked with asterisk).
///
/// # Errors
///
/// Returns an error if the networksetup command fails or output cannot be parsed.
fn active_networks() -> Result<Vec<String>, Box<dyn Error>> {
    let mut command = Command::new("networksetup");
    command.arg("-listallnetworkservices");

    let output = command_stdout(&mut command, "list network services")?;
    Ok(parse_network_services(&output))
}

/// Enables public DNS servers on all active network interfaces
///
/// Sets CloudFlare (1.1.1.1, 2606:4700:4700::1111) and Google
/// (8.8.4.4, 2001:4860:4860::8844) DNS servers.
///
/// # Errors
///
/// Returns an error if DNS configuration update fails.
fn enable_pub_dns() -> Result<(), Box<dyn Error>> {
    apply_dns_config(
        PUBLIC_DNS,
        |network| format!("Enable public DNS servers {PUBLIC_DNS:?} on device '{network}'"),
        |manual_dns| match manual_dns {
            ManualDns::Servers(current_dns) => PUBLIC_DNS
                .iter()
                .all(|&public_dns| current_dns.iter().any(|dns| dns == public_dns)),
            ManualDns::Dhcp => false,
        },
        |manual_dns| format!(" Not OK: (Expected all {PUBLIC_DNS:?}, but got {manual_dns:?})"),
    )
}

/// Reverts to DHCP-assigned DNS servers on all active network interfaces
///
/// Clears manually configured DNS settings, allowing the DHCP server to
/// provide DNS configuration.
///
/// # Errors
///
/// Returns an error if DNS configuration update fails.
fn enable_dhcp_dns() -> Result<(), Box<dyn Error>> {
    apply_dns_config(
        &["empty"],
        |network| format!("Revert to DHCP-assigned DNS servers on device '{network}' "),
        |manual_dns| matches!(manual_dns, ManualDns::Dhcp),
        |manual_dns| format!(" Not OK (DNS servers still defined: {manual_dns:?})"),
    )
}

/// Helper function to apply DNS configuration to all active networks
///
/// Provides a reusable pattern for DNS updates with validation.
///
/// # Arguments
///
/// * `dns_servers` - DNS server addresses to set (or ["empty"] for DHCP)
/// * `format_msg` - Closure to format the status message for each network
/// * `validate` - Closure to validate DNS configuration was applied correctly
/// * `error_msg` - Closure to format error message if validation fails
///
/// # Errors
///
/// Returns an error if network commands fail or DNS update fails.
fn apply_dns_config<F, V, E>(
    dns_servers: &[&str],
    format_msg: F,
    validate: V,
    error_msg: E,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) -> String,
    V: Fn(&ManualDns) -> bool,
    E: Fn(&ManualDns) -> String,
{
    let networks = active_networks()?;

    for network in networks {
        print!("{}", format_msg(&network));

        update_dns_servers(&network, dns_servers)?;

        let manual_dns = manual_dns_of_network(&network)?;

        if validate(&manual_dns) {
            println!("{}", " OK".green());
        } else {
            let message = error_msg(&manual_dns);
            println!("{}", message.red());
            return Err(format!("DNS validation failed for '{network}': {}", message.trim()).into());
        }
    }

    Ok(())
}

/// Updates DNS servers for a specific network interface
///
/// Uses `sudo networksetup -setdnsservers` to modify DNS configuration.
/// Requires sudo privileges.
///
/// # Arguments
///
/// * `network` - Name of the network interface
/// * `dns_args` - DNS server addresses (or ["empty"] to clear)
///
/// # Errors
///
/// Returns an error if the networksetup command fails.
fn update_dns_servers(network: &str, dns_args: &[&str]) -> Result<(), Box<dyn Error>> {
    let output = Command::new("sudo")
        .arg("networksetup")
        .arg("-setdnsservers")
        .arg(network)
        .args(dns_args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Failed to update DNS servers with status {}: {}",
            output.status,
            stderr.trim()
        )
        .into());
    }

    Ok(())
}

/// Gets manually configured DNS servers for a network interface
///
/// Uses `networksetup -getdnsservers` to retrieve DNS configuration.
///
/// # Arguments
///
/// * `network` - Name of the network interface
///
/// # Returns
///
/// Manual DNS state for the requested network service.
///
/// # Errors
///
/// Returns an error if the networksetup command fails.
fn manual_dns_of_network(network: &str) -> Result<ManualDns, Box<dyn Error>> {
    let mut command = Command::new("networksetup");
    command.arg("-getdnsservers").arg(network);

    let output = command_stdout(&mut command, &format!("get DNS servers for '{network}'"))?;
    Ok(parse_manual_dns_output(&output))
}

/// Gets current DNS servers for a network interface
///
/// Checks manually configured DNS first, then falls back to DHCP-assigned
/// DNS servers from `scutil --dns` if no manual configuration exists.
///
/// # Arguments
///
/// * `network` - Name of the network interface
///
/// # Returns
///
/// Vector of currently active DNS server addresses.
///
/// # Errors
///
/// Returns an error if system commands fail.
fn current_dns_servers(network: &str) -> Result<Vec<String>, Box<dyn Error>> {
    match manual_dns_of_network(network)? {
        ManualDns::Servers(dns_servers) => Ok(dns_servers),
        ManualDns::Dhcp => {
            let mut command = Command::new("scutil");
            command.arg("--dns");

            let output = command_stdout(&mut command, "read DHCP DNS configuration")?;
            Ok(parse_scutil_dns_servers(&output))
        }
    }
}

fn command_stdout(command: &mut Command, description: &str) -> Result<String, Box<dyn Error>> {
    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{description} failed with status {}: {}", output.status, stderr.trim()).into());
    }

    Ok(str::from_utf8(&output.stdout)?.to_string())
}

fn parse_network_services(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("An asterisk"))
        .filter(|line| !line.starts_with('*'))
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_manual_dns_output(output: &str) -> ManualDns {
    let dns_servers: Vec<String> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if dns_servers
        .iter()
        .any(|dns| dns.starts_with("There aren't any DNS Servers set on"))
    {
        ManualDns::Dhcp
    } else {
        ManualDns::Servers(dns_servers)
    }
}

fn parse_scutil_dns_servers(scutil_output: &str) -> Vec<String> {
    let mut dns_servers = Vec::new();
    let mut seen = HashSet::new();

    for line in scutil_output.lines() {
        if line.trim().starts_with("nameserver[") {
            if let Some((_, ip_part)) = line.split_once(':') {
                let ip = ip_part.trim().to_string();
                if seen.insert(ip.clone()) {
                    dns_servers.push(ip);
                }
            }
        }
    }

    dns_servers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_network_services_skips_legend_and_disabled_services() {
        let output = "\
An asterisk (*) denotes that a network service is disabled.
Wi-Fi
*Bluetooth PAN
USB 10/100/1000 LAN
";

        assert_eq!(
            parse_network_services(output),
            vec!["Wi-Fi".to_string(), "USB 10/100/1000 LAN".to_string()]
        );
    }

    #[test]
    fn parse_manual_dns_output_detects_dhcp_state() {
        let output = "There aren't any DNS Servers set on Wi-Fi.\n";

        assert_eq!(parse_manual_dns_output(output), ManualDns::Dhcp);
    }

    #[test]
    fn parse_manual_dns_output_returns_servers() {
        let output = "\
1.1.1.1
2606:4700:4700::1111
8.8.4.4
";

        assert_eq!(
            parse_manual_dns_output(output),
            ManualDns::Servers(vec![
                "1.1.1.1".to_string(),
                "2606:4700:4700::1111".to_string(),
                "8.8.4.4".to_string()
            ])
        );
    }

    #[test]
    fn parse_scutil_dns_servers_deduplicates_preserving_order() {
        let output = "\
DNS configuration

resolver #1
  nameserver[0] : 192.168.1.1
  nameserver[1] : 1.1.1.1

resolver #2
  nameserver[0] : 1.1.1.1
  nameserver[1] : 2606:4700:4700::1111
";

        assert_eq!(
            parse_scutil_dns_servers(output),
            vec![
                "192.168.1.1".to_string(),
                "1.1.1.1".to_string(),
                "2606:4700:4700::1111".to_string()
            ]
        );
    }
}
