# wakeup

## Usage

```
wakeup 
Wake up a host in the network with (a) magic (packet)

USAGE:
    wakeup [OPTIONS] <hostname|--mac <MAC_ADDRESS>>

ARGS:
    <hostname>    the name of the host you want to wake up

OPTIONS:
    -h, --help                   Print help information
    -i, --use-ip <IP_ADDRESS>    Use specific ip address instead of local broadcast [default:
                                 255.255.255.255]
    -m, --mac <MAC_ADDRESS>      the mac address of the host you want to wake up
    -p, --use-port <PORT>        Use specific port instead of default [default: 9]
```
## Config files
On linux, `~/.config/wakeup/config.toml` is used when looking for a config file.

Example config.toml:

```
hosts = [
    { name= "my_host", mac_addresses = ["aa:bb:cc:dd:ee:ff"] },
    { name = "my_other_host", mac_addresses = ["00:11:22:33:ff:ff", "00:11:22:33:44:ff"] },
]

```
