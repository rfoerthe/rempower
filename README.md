# Rempower
Rust-empowered command line tools for macOS

## Provided Tools
All provided tools are organized as sub commands of `rem`.

### dns
`dns` lets you easily switch between public DNS servers and those assigned by your local network's DHCP server.
If you're using a DNS forwarder in your local network—often provided by your router—you might occasionally 
want to bypass it and access public DNS servers directly. 
In my case, I needed this because I had enabled DNS filtering on my router to block 
blacklisted and advertising-related domains, which I sometimes want to bypass. 

#### Examples

Lists active DNS servers:
```zsh
rem dns -l
```

Use public DNS servers from CloudFlare and Google exclusively
```zsh
rem dns --pub 
```

Revert to DNS servers assigned by the DHCP server
```zsh
rem dns --dhcp
```

## Shell Completions

`rem` supports shell completions for various shells to help you use commands and options more efficiently.

### Supported Shells

- **bash**
- **elvish**
- **fish**
- **powershell**
- **zsh**

### Installation

#### Bash
```bash
# Temporarily for current session
eval "$(rem completions bash)"

# Permanently - add this to your ~/.bashrc
echo 'eval "$(rem completions bash)"' >> ~/.bashrc
```

#### Zsh
```zsh
# Temporarily for current session
eval "$(rem completions zsh)"

# Permanently - add this to your ~/.zshrc
echo 'eval "$(rem completions zsh)"' >> ~/.zshrc

# Or save to a completion directory
rem completions zsh > ~/.zsh/completions/_rem
```

#### Fish
```shell
# Generate completion file
rem completions fish > ~/.config/fish/completions/rem.fish
```

#### PowerShell
```shell
# Temporarily for current session
rem completions powershell | Out-String | Invoke-Expression

# Permanently - add this to your PowerShell profile
Add-Content -Path $PROFILE -Value 'rem completions powershell | Out-String | Invoke-Expression'
```

#### Elvish
```shell
# Generate completion file
rem completions elvish > ~/.config/elvish/completions/rem.elv
```

### Usage
After installation, you can use the Tab key to:
- Complete available commands
- Show options and flags
- Complete arguments

Example:

```shell
rem <TAB>          # Shows available commands
rem dns <TAB>      # Shows DNS options
rem dns --<TAB>    # Shows available flags
```

### Notes
- Start a new shell session or reload your configuration after installation
- If you encounter issues, verify that your shell supports completions
- For zsh, ensure that `compinit` is enabled in your `.zshrc`
