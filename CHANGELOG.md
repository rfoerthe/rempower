# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.4] - 2025-10-06

### Fixed
- **Error handling in DNS operations**: `update_dns_servers` now properly checks if the sudo command succeeded and returns detailed error messages when DNS configuration fails
- **DNS validation logic**: Public DNS validation now correctly verifies that ALL configured DNS servers match the expected list, not just any one of them
- **Error messages**: Improved error message clarity to show both expected and actual DNS configurations

### Changed
- **Parameter types**: Changed function parameters from `&String` to `&str` for more idiomatic Rust code (affects `update_dns_servers` and `manual_dns_of_network`)
- **Code structure**: Refactored `enable_pub_dns` and `enable_dhcp_dns` to use a shared `apply_dns_config` helper function, eliminating code duplication

### Removed
- **Unused code**: Removed unused `is_pub_dns()` method from `DnsArgs` implementation
- **Empty module**: Removed empty `common.rs` module and its reference from `lib.rs`

### Added
- **Documentation**: Added comprehensive doc comments to the public `perform` function explaining its purpose, arguments, and error handling

## [0.5.3] - 2025-08-27

### Changed
- Split `switch_dns` into separate `enable_dhcp_dns` and `enable_pub_dns` functions for better code organization

## [0.5.2] - 2025-07-17

### Added
- Extended DNS list with IPv6 addresses for CloudFlare and Google DNS servers

## [0.5.1] - 2025-07-16

### Changed
- Updated CLI `about` to include dynamic version number from Cargo.toml

### Fixed
- Corrected wording in README and updated code block syntax for Zsh examples

## [0.5.0] - 2025-07-15

### Added
- Initial release with DNS switching functionality
- Shell completions support for bash, elvish, fish, powershell, and zsh
