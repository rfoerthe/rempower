# Rempower

Rust-empowered command line tools for macOS.

Rempower installs the `rem` binary. Each utility is exposed as a subcommand.

## Requirements

- macOS
- Rust toolchain 1.88, managed through `rust-toolchain.toml`
- `sudo` privileges for commands that modify system settings

## Installation

Build and install the binary from this repository:

```shell
cargo install --path .
```

For local development, run the binary without installing it:

```shell
cargo run -- <command>
```

## Usage

```shell
rem dns --list
rem dns --pub
rem dns --dhcp
rem completions zsh
```

Show available commands:

```shell
rem --help
```

Show DNS command options:

```shell
rem dns --help
```

## Tools

### `dns`

Switch DNS settings for active macOS network services.

This is useful when you normally use DNS servers assigned by DHCP, such as a router or local DNS forwarder, but occasionally want to bypass them with public DNS servers.

Options:

- `rem dns --list`: list currently active DNS servers
- `rem dns --pub`: set public DNS servers manually
- `rem dns --dhcp`: clear manually configured DNS servers and use DHCP-provided DNS again

`--pub` configures these public DNS servers:

- Cloudflare IPv4: `1.1.1.1`
- Cloudflare IPv6: `2606:4700:4700::1111`
- Google IPv4: `8.8.4.4`
- Google IPv6: `2001:4860:4860::8844`

Changing DNS settings uses macOS `networksetup` through `sudo`.

## Shell Completions

`rem` can generate shell completions for:

- bash
- elvish
- fish
- powershell
- zsh

### Bash

```bash
# Current session
eval "$(rem completions bash)"

# Permanent setup
echo 'eval "$(rem completions bash)"' >> ~/.bashrc
```

### Zsh

```shell
# Current session
eval "$(rem completions zsh)"

# Permanent setup
echo 'eval "$(rem completions zsh)"' >> ~/.zshrc

# Or save to a completion directory
rem completions zsh > ~/.zsh/completions/_rem
```

For zsh completion files, make sure `compinit` is enabled in your shell configuration.

### Fish

```shell
rem completions fish > ~/.config/fish/completions/rem.fish
```

### PowerShell

```shell
# Current session
rem completions powershell | Out-String | Invoke-Expression

# Permanent setup
Add-Content -Path $PROFILE -Value 'rem completions powershell | Out-String | Invoke-Expression'
```

### Elvish

```shell
rem completions elvish > ~/.config/elvish/completions/rem.elv
```

## Development

Run the standard checks before committing:

```shell
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The GitHub Actions workflow in `.github/workflows/rust.yml` runs formatting, check, clippy, build, and tests for pushes and pull requests targeting `master`.

## License

See `LICENSE`.
