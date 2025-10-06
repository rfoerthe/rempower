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
use std::error::Error;
use std::process::Command;

/// Public DNS servers (CloudFlare and Google with IPv4 and IPv6)
const PUBLIC_DNS: &[&str] = &["1.1.1.1", "2606:4700:4700::1111", "8.8.4.4", "2001:4860:4860::8844"];

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
        println!("{:>30} : {:?}", network, dns_servers);
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
    let output = Command::new("networksetup").arg("-listallnetworkservices").output()?;

    let output_str = str::from_utf8(&output.stdout)?.trim();
    let active_network_services: Vec<String> = output_str
        .lines()
        .filter(|line| !line.contains("An asterisk") && !line.contains("(*)"))
        .map(|line| line.trim().to_string())
        .collect();

    Ok(active_network_services)
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
        |network| format!("Enable public DNS servers {:?} on device '{}'", PUBLIC_DNS, network),
        |current_dns| {
            PUBLIC_DNS
                .iter()
                .all(|&public_dns| current_dns.iter().any(|dns| dns == public_dns))
        },
        |current_dns| {
            format!(
                " Not OK: (Expected all {:?}, but got {:?})",
                PUBLIC_DNS, current_dns
            )
        },
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
        |network| format!("Revert to DHCP-assigned DNS servers on device '{}' ", network),
        |current_dns| {
            current_dns
                .iter()
                .any(|dns| dns.contains("There aren't any DNS Servers set on"))
        },
        |current_dns| format!(" Not OK (DNS servers still defined: {:?})", current_dns),
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
    V: Fn(&[String]) -> bool,
    E: Fn(&[String]) -> String,
{
    let networks = active_networks()?;

    for network in networks {
        print!("{}", format_msg(&network));

        update_dns_servers(&network, dns_servers)?;

        let current_dns = manual_dns_of_network(&network)?;

        if validate(&current_dns) {
            println!("{}", " OK".green());
        } else {
            println!("{}", error_msg(&current_dns).red());
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
        .arg(&network)
        .args(dns_args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to update DNS servers: {}", stderr).into());
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
/// Vector of DNS server addresses, or a message indicating no manual DNS is set.
///
/// # Errors
///
/// Returns an error if the networksetup command fails.
fn manual_dns_of_network(network: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let dns_output = Command::new("networksetup")
        .arg("-getdnsservers")
        .arg(&network)
        .output()?;

    let dns_vec = str::from_utf8(&dns_output.stdout)?
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    Ok(dns_vec)
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
    let dns_result = manual_dns_of_network(network)?;

    // If DNS servers aren't configured manually, check DHCP-DNS
    if dns_result
        .iter()
        .any(|dns| dns.contains("There aren't any DNS Servers set on"))
    {
        let scutil_output = Command::new("scutil").arg("--dns").output()?;
        let scutil_str = str::from_utf8(&scutil_output.stdout)?;
        let dns_servers = extract_dns_from_scutil(scutil_str)?;
        if dns_servers.is_empty() {
            Ok(dns_result)
        } else {
            Ok(dns_servers)
        }
    } else {
        Ok(dns_result)
    }
}

/// Extracts DNS server addresses from `scutil --dns` output
///
/// Parses the output to find nameserver entries and deduplicate them.
///
/// # Arguments
///
/// * `scutil_output` - Output from `scutil --dns` command
///
/// # Returns
///
/// Vector of unique DNS server addresses found in the output.
///
/// # Errors
///
/// Returns an error if parsing fails.
fn extract_dns_from_scutil(scutil_output: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut dns_servers = Vec::new();

    for line in scutil_output.lines() {
        if line.trim().starts_with("nameserver[") {
            if let Some((_, ip_part)) = line.split_once(':') {
                let ip = ip_part.trim().to_string();
                if !dns_servers.contains(&ip) {
                    dns_servers.push(ip);
                }
            }
        }
    }

    Ok(dns_servers)
}
