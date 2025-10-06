# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rempower is a Rust CLI tool providing macOS system utilities. The main binary `rem` uses a subcommand architecture where each tool is implemented as a subcommand.

## Build & Development Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run the tool
cargo run -- <subcommand> [args]

# Run with specific arguments (e.g., list DNS)
cargo run -- dns -l

# Check code without building
cargo check

# Format code (uses max_width = 120 from rustfmt.toml)
cargo fmt

# Run clippy
cargo clippy
```

## Architecture

### CLI Structure

The project uses `clap` with derive macros for CLI parsing:

- **src/bin/rem.rs** - Main entry point, parses CLI args and dispatches to subcommands
- **src/cli.rs** - Defines CLI structure using clap derive macros
  - `Cli` struct: Root command parser
  - `Commands` enum: All available subcommands
  - Command-specific args structs (e.g., `DnsArgs`)
- **src/subcommands/** - Each subcommand module implements its functionality
- **src/lib.rs** - Library root exposing public modules

### Adding New Subcommands

1. Add variant to `Commands` enum in src/cli.rs
2. Create args struct if needed (following `DnsArgs` pattern)
3. Create subcommand module in src/subcommands/
4. Export module in src/subcommands.rs
5. Handle command in main() match statement in src/bin/rem.rs

### DNS Subcommand Architecture

The DNS subcommand (src/subcommands/dns.rs) demonstrates the pattern:

- Uses macOS `networksetup` command to configure DNS servers
- Uses `scutil --dns` to read DHCP-assigned DNS servers
- Public DNS servers defined in `PUBLIC_DNS` constant: CloudFlare (1.1.1.1, 2606:4700:4700::1111) and Google (8.8.4.4, 2001:4860:4860::8844)
- Helper function pattern: `apply_dns_config()` abstracts common DNS update logic with validation callbacks
- All network operations use `std::process::Command` to execute system commands

## macOS System Integration

This tool requires macOS and uses these system commands:
- `networksetup -listallnetworkservices` - List network interfaces
- `networksetup -setdnsservers` - Configure DNS (requires sudo)
- `networksetup -getdnsservers` - Get manually configured DNS
- `scutil --dns` - Get all DNS configuration including DHCP

## Shell Completions

The tool generates shell completions using `clap_complete`:
- Supported shells: bash, zsh, fish, powershell, elvish
- Generated via: `rem completions <shell>`
- Implementation in src/bin/rem.rs using `generate()` function

## Toolchain

- Rust edition: 2024
- Toolchain version: 1.88 (defined in rust-toolchain.toml)
- Required components: rustfmt, clippy
