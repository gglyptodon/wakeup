# wakeup
[![Rust](https://github.com/gglyptodon/wakeup/actions/workflows/rust.yml/badge.svg)](https://github.com/gglyptodon/wakeup/actions/workflows/rust.yml)
## Usage

```
wakeup 
Wake up a host in the network with (a) magic (packet)

USAGE:
    wakeup [OPTIONS] <hostname|--mac <MAC_ADDRESS>>

ARGS:
    <hostname>    Name of the host you want to wake up

OPTIONS:
    -h, --help                   Print help information
    -i, --use-ip <IP_ADDRESS>    Use specific ip address instead of local broadcast [default:
                                 255.255.255.255]
    -m, --mac <MAC_ADDRESS>      MAC address of the host you want to wake up
    -p, --use-port <PORT>        Use specific port instead of default [default: 9]
```

## Config files
On linux, `~/.config/wakeup/config.toml` is used when looking for a config file.

On Mac: `~/Library/Application Suppport/dev.gglyptodon.wakeup/config.toml` is used when looking for a config file.

On Windows: `{FOLDERID_RoamingAppData}`, e.g. `C:\Users\<USERNAME>\AppData\Roaming\gglyptodon\wakeup\config\config.toml` is used when looking for a config file.

Example `config.toml`:

```
hosts = [
    { name= "my_host", mac_addresses = ["aa:bb:cc:dd:ee:ff"] },
    { name = "my_other_host", mac_addresses = ["00:11:22:33:ff:ff", "00:11:22:33:44:ff"] },
    { name = "all_my_hosts", mac_addresses = ["00:11:22:33:ff:ff", "00:11:22:33:44:ff", "aa:bb:cc:dd:ee:ff"] },
]
```
