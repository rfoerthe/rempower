use clap::Parser;
use colored::*;
use rempower::cli::{Cli, Commands};
use std::error::Error;
use std::process::Command;
use std::str;

const PUBLIC_DNS: &[&str] = &["1.1.1.1", "8.8.4.4"];

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dns(args) => _set_pub_dns(args.is_pub_dns())?,
    }

    Ok(())
}

/// Return names of active network interfaces as Vec<String>
fn active_networks() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()?;

    let output_str = str::from_utf8(&output.stdout)?.trim();
    let active_network_services: Vec<String> = output_str
        .lines()
        .filter(|line| !line.contains("An asterisk") && !line.contains("(*)"))
        .map(|line| line.trim().to_string())
        .collect();

    Ok(active_network_services)
}

fn _set_pub_dns(enable_pub_dns: bool) -> Result<(), Box<dyn Error>> {
    let networks = active_networks()?;

    if enable_pub_dns {
        for network in networks {
            print!(
                "Enable public DNS servers {:?} on device '{}'",
                PUBLIC_DNS, network
            );

            update_dns_servers(&network, PUBLIC_DNS)?;

            let current_dns = dns_of_network(&network)?;

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
            print!(
                "Revert to DHCP-assigned DNS servers on device '{}' ",
                network
            );

            update_dns_servers(&network, &["empty"])?;

            let current_dns = dns_of_network(&network)?;

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

fn dns_of_network(network: &String) -> Result<Vec<String>, Box<dyn Error>> {
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
