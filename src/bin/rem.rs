use clap::Parser;
use colored::*;
use rempower::cli::{Cli, Commands};
use std::collections::HashSet;
use std::error::Error;
use std::process::Command;
use std::str;

const PUBLIC_DNS: &[&str] = &["1.1.1.1", "8.8.4.4"];

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dns(args) => _set_pub_dns(args.is_pub_dns())?
    }

    Ok(())
}


/// Return names of active network interfaces as Vec<String>
fn active_networks() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()?;

    let output_str = str::from_utf8(&output.stdout)?;
    let mut active_network_services = Vec::new();

    for line in output_str.trim().lines() {
        if !line.starts_with("An asterisk") && !line.starts_with("*") {
            active_network_services.push(line.to_string());
        }
    }

    Ok(active_network_services)
}


fn _set_pub_dns(enable_pub_dns: bool) -> Result<(), Box<dyn Error>> {
    let networks = active_networks()?;

    if enable_pub_dns {
        for network in networks {
            print!("Enable Public DNS {:?} on device '{}'", PUBLIC_DNS, network);

            let mut cmd = Command::new("sudo");
            cmd.arg("networksetup")
                .arg("-setdnsservers")
                .arg(&network);

            for dns in PUBLIC_DNS {
                cmd.arg(dns);
            }

            cmd.output()?;

            let dns_output = Command::new("networksetup")
                .arg("-getdnsservers")
                .arg(&network)
                .output()?;

            let dns_str = str::from_utf8(&dns_output.stdout)?;
            let dns_servers: HashSet<&str> = dns_str.split_whitespace().collect();
            let public_dns_set: HashSet<&str> = PUBLIC_DNS.iter().copied().collect();

            if !public_dns_set.is_disjoint(&dns_servers) {
                println!("{}", " OK".green());
            } else {
                println!("{}", format!(" Not OK: {}", dns_str).red());
            }
        }
    } else {
        for network in networks {
            print!("Reset DNS on device '{}' and activating DNS servers configured in router", network);

            Command::new("sudo")
                .arg("networksetup")
                .arg("-setdnsservers")
                .arg(&network)
                .arg("empty")
                .output()?;

            let dns_output = Command::new("networksetup")
                .arg("-getdnsservers")
                .arg(&network)
                .output()?;

            let dns_str = str::from_utf8(&dns_output.stdout)?;

            if dns_str.starts_with("There aren't any DNS Servers set on") {
                println!("{}", " OK".green());
            } else {
                println!("{}", format!(" Not OK: {}", dns_str).red());
            }
        }
    }

    Ok(())
}