# Rempower
Rust empowered tools for macOS

## Provided Tools
All provided tools are organized as sub commands of `rem`.

### dns
`dns` lets you easily switch between public DNS servers and those assigned by your local network's DHCP server.
If you're using a DNS forwarder in your local network—often provided by your router—you might occasionally 
want to bypass it and access public DNS servers directly. 
In my case, I needed this because I had enabled DNS filtering on my router to block 
blacklisted and advertising-related domains, which I sometimes want to bypass. 

#### Examples

List active DNS servers:
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




