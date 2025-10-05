use crate::cli::DnsArgs;
use colored::Colorize;
use std::error::Error;
use std::process::Command;

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

fn print_current_dns() -> Result<(), Box<dyn Error>> {
    let networks = active_networks()?;

    for network in networks {
        let dns_servers = current_dns_servers(&network)?;
        println!("{:>30} : {:?}", network, dns_servers);
    }

    Ok(())
}

/// Return names of active network interfaces as Vec<String>
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
