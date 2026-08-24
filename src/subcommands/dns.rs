//! DNS configuration subcommand
//!
//! Provides functionality to configure DNS servers on macOS network interfaces.
//! Supports switching between public DNS servers (CloudFlare and Google) and
//! DHCP-assigned DNS servers.
//!
//! # System Requirements
//!
//! - macOS operating system
//! - Sudo privileges for modifying DNS settings
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
const NETWORKSETUP: &str = "/usr/sbin/networksetup";
const SCUTIL: &str = "/usr/sbin/scutil";
const SUDO: &str = "/usr/bin/sudo";

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
    let mut command = Command::new(NETWORKSETUP);
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
        pub_dns_valid,
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

/// True when the given manual DNS state already contains every public DNS server.
fn pub_dns_valid(manual_dns: &ManualDns) -> bool {
    match manual_dns {
        ManualDns::Servers(current_dns) => PUBLIC_DNS
            .iter()
            .all(|&public_dns| current_dns.iter().any(|dns| dns == public_dns)),
        ManualDns::Dhcp => false,
    }
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
    let mut command = Command::new(SUDO);
    command
        .arg(NETWORKSETUP)
        .arg("-setdnsservers")
        .arg(network)
        .args(dns_args);

    command_stdout(&mut command, &format!("update DNS servers on '{network}'"))?;

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
    let mut command = Command::new(NETWORKSETUP);
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
            let mut command = Command::new(SCUTIL);
            command.arg("--dns");

            let output = command_stdout(&mut command, "read DHCP DNS configuration")?;
            Ok(parse_scutil_dns_servers(&output))
        }
    }
}

fn command_stdout(command: &mut Command, description: &str) -> Result<String, Box<dyn Error>> {
    let output = exec(command)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{description} failed with status {}: {}", output.status, stderr.trim()).into());
    }

    Ok(str::from_utf8(&output.stdout)?.to_string())
}

/// Executes a command, delegating to the stubbed runner when testing.
fn exec(command: &mut Command) -> Result<std::process::Output, Box<dyn Error>> {
    #[cfg(test)]
    {
        if let Some(front) = tests::next_output() {
            return Ok(front);
        }
    }
    Ok(command.output()?)
}

fn trimmed_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_network_services(output: &str) -> Vec<String> {
    trimmed_lines(output)
        .into_iter()
        .filter(|line| !line.starts_with("An asterisk"))
        .filter(|line| !line.starts_with('*'))
        .collect()
}

fn parse_manual_dns_output(output: &str) -> ManualDns {
    let dns_servers = trimmed_lines(output);

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
        let line = line.trim();

        if !line.starts_with("nameserver[") {
            continue;
        }

        if let Some((_, ip_part)) = line.split_once(':') {
            let ip = ip_part.trim().to_string();
            if seen.insert(ip.clone()) {
                dns_servers.push(ip);
            }
        }
    }

    dns_servers
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::os::unix::process::ExitStatusExt;
    use std::sync::Mutex;

    static QUEUE: Mutex<RefCell<Vec<std::process::Output>>> = Mutex::new(RefCell::new(Vec::new()));
    static SERIAL: Mutex<()> = Mutex::new(());

    /// Pops the next queued stubbed output (FIFO), if any.
    pub(super) fn next_output() -> Option<std::process::Output> {
        let guard = QUEUE.lock().unwrap();
        let mut queue = guard.borrow_mut();

        if !queue.is_empty() {
            return Some(queue.remove(0));
        }

        None
    }

    fn push_output(stdout: &str, status: u8, stderr: &str) {
        QUEUE.lock().unwrap().borrow_mut().push(std::process::Output {
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
            status: std::process::ExitStatus::from_raw(i32::from(status)),
        });
    }

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

    #[test]
    fn parse_network_services_returns_empty_for_blank_output() {
        assert!(parse_network_services("\n\n   \n").is_empty());
    }

    #[test]
    fn parse_network_services_skips_legend_line() {
        let output = "An asterisk (*) denotes that a network service is disabled.\n";

        assert!(parse_network_services(output).is_empty());
    }

    #[test]
    fn parse_manual_dns_output_empty_output_returns_servers() {
        assert_eq!(parse_manual_dns_output(""), ManualDns::Servers(vec![]));
    }

    #[test]
    fn parse_scutil_dns_servers_ignores_non_nameserver_lines() {
        let output = "\
DNS configuration
  domain   : example.com

resolver #1
  nameserver[0] : 192.168.1.1
";

        assert_eq!(parse_scutil_dns_servers(output), vec!["192.168.1.1".to_string()]);
    }

    #[test]
    fn pub_dns_valid_false_for_dhcp() {
        assert!(!pub_dns_valid(&ManualDns::Dhcp));
    }

    #[test]
    fn pub_dns_valid_true_when_all_public_servers_present() {
        let servers = ManualDns::Servers(PUBLIC_DNS.iter().map(|s| s.to_string()).collect());

        assert!(pub_dns_valid(&servers));
    }

    #[test]
    fn pub_dns_valid_false_when_server_missing() {
        let servers = ManualDns::Servers(vec!["1.1.1.1".to_string(), "2606:4700:4700::1111".to_string()]);

        assert!(!pub_dns_valid(&servers));
    }

    #[test]
    fn active_networks_parses_stubbed_output() {
        let _serial = SERIAL.lock().unwrap();
        push_output("Wi-Fi\n*Disabled\nUSB LAN\n", 0, "");

        assert_eq!(
            active_networks().unwrap(),
            vec!["Wi-Fi".to_string(), "USB LAN".to_string()]
        );
    }

    #[test]
    fn active_networks_propagates_command_error() {
        let _serial = SERIAL.lock().unwrap();
        push_output("", 1, "boom");

        let err = active_networks().unwrap_err();
        assert!(err.to_string().contains("list network services failed with status"));
    }

    #[test]
    fn manual_dns_of_network_parses_servers_and_dhcp() {
        let _serial = SERIAL.lock().unwrap();
        push_output("1.1.1.1\n8.8.4.4\n", 0, "");

        assert_eq!(
            manual_dns_of_network("Wi-Fi").unwrap(),
            ManualDns::Servers(vec!["1.1.1.1".to_string(), "8.8.4.4".to_string()])
        );

        push_output("There aren't any DNS Servers set on Wi-Fi.\n", 0, "");
        assert_eq!(manual_dns_of_network("Wi-Fi").unwrap(), ManualDns::Dhcp);
    }

    #[test]
    fn current_dns_servers_uses_manual_when_present() {
        let _serial = SERIAL.lock().unwrap();
        push_output("1.1.1.1\n", 0, "");

        assert_eq!(current_dns_servers("Wi-Fi").unwrap(), vec!["1.1.1.1".to_string()]);
    }

    #[test]
    fn current_dns_servers_falls_back_to_dhcp() {
        let _serial = SERIAL.lock().unwrap();
        push_output("There aren't any DNS Servers set on Wi-Fi.\n", 0, "");
        push_output("resolver #1\n  nameserver[0] : 192.168.1.1\n", 0, "");

        assert_eq!(current_dns_servers("Wi-Fi").unwrap(), vec!["192.168.1.1".to_string()]);
    }

    #[test]
    fn update_dns_servers_succeeds() {
        let _serial = SERIAL.lock().unwrap();
        push_output("", 0, "");

        assert!(update_dns_servers("Wi-Fi", PUBLIC_DNS).is_ok());
    }

    #[test]
    fn update_dns_servers_propagates_error() {
        let _serial = SERIAL.lock().unwrap();
        push_output("", 1, "permission denied");

        let err = update_dns_servers("Wi-Fi", PUBLIC_DNS).unwrap_err();
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn apply_dns_config_validates_each_network() {
        let _serial = SERIAL.lock().unwrap();
        // active_networks + update + manual for Wi-Fi, then update + manual for USB LAN.
        push_output("Wi-Fi\nUSB LAN\n", 0, "");
        push_output("", 0, "");
        push_output("1.1.1.1\n2606:4700:4700::1111\n8.8.4.4\n2001:4860:4860::8844\n", 0, "");
        push_output("", 0, "");
        push_output("1.1.1.1\n2606:4700:4700::1111\n8.8.4.4\n2001:4860:4860::8844\n", 0, "");

        let result = apply_dns_config(
            PUBLIC_DNS,
            |network| network.to_string(),
            pub_dns_valid,
            |manual_dns| format!(" bad: {manual_dns:?}"),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn apply_dns_config_reports_validation_failure() {
        let _serial = SERIAL.lock().unwrap();
        push_output("Wi-Fi\n", 0, "");
        push_output("", 0, "");
        push_output("1.1.1.1\n", 0, "");

        let result = apply_dns_config(
            PUBLIC_DNS,
            |network| network.to_string(),
            pub_dns_valid,
            |manual_dns| format!(" bad: {manual_dns:?}"),
        );

        assert!(result.is_err());
    }

    #[test]
    fn perform_dispatches_subcommands() {
        let _serial = SERIAL.lock().unwrap();
        // Empty network list keeps the test platform-independent (no real DNS commands).
        push_output("\n", 0, "");

        perform(DnsArgs {
            pub_dns: false,
            dhcp: false,
            list: true,
        })
        .unwrap();
    }
}
