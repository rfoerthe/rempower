use crate::cli::DnsArgs;
use colored::Colorize;
use std::error::Error;
use std::process::Command;

const PUBLIC_DNS: &[&str] = &["1.1.1.1", "2606:4700:4700::1111", "8.8.4.4", "2001:4860:4860::8844"];

pub fn perform(args: DnsArgs) -> Result<(), Box<dyn Error>> {
    if args.dhcp {
        switch_dns(false)?;
    } else if args.pub_dns {
        switch_dns(true)?;
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

fn switch_dns(enable_pub_dns: bool) -> Result<(), Box<dyn Error>> {
    let networks = active_networks()?;

    if enable_pub_dns {
        for network in networks {
            print!("Enable public DNS servers {:?} on device '{}'", PUBLIC_DNS, network);

            update_dns_servers(&network, PUBLIC_DNS)?;

            let current_dns = manual_dns_of_network(&network)?;

            if PUBLIC_DNS
                .iter()
                .any(|&public_dns| current_dns.iter().any(|dns| dns == public_dns))
            {
                println!("{}", " OK".green());
            } else {
                println!(
                    "{}",
                    format!(
                        " Not OK: (These configured DNS servers are not expected {:?})",
                        current_dns
                    )
                        .red()
                );
            }
        }
    } else {
        for network in networks {
            print!("Revert to DHCP-assigned DNS servers on device '{}' ", network);

            update_dns_servers(&network, &["empty"])?;

            let current_dns = manual_dns_of_network(&network)?;

            if current_dns
                .iter()
                .any(|dns| dns.contains("There aren't any DNS Servers set on"))
            {
                println!("{}", " OK".green());
            } else {
                println!(
                    "{}",
                    format!(" Not OK (DNS servers still defined: {:?})", current_dns).red()
                );
            }
        }
    }

    Ok(())
}

fn update_dns_servers(network: &String, dns_args: &[&str]) -> Result<(), Box<dyn Error>> {
    Command::new("sudo")
        .arg("networksetup")
        .arg("-setdnsservers")
        .arg(&network)
        .args(dns_args)
        .output()?;
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
