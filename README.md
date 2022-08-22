# wakeup

## Usage

```
wakeup 
Wake up a machine in the network with (a) magic (packet)

USAGE:
    wakeup [OPTIONS] <hostname|mac_address>

ARGS:
    <hostname>       the name of the machine you want to wake up
    <mac_address>    the mac address of the machine you want to wake up

OPTIONS:
        --debug                  Print debug output to stdout
    -h, --help                   Print help information
    -i, --use-ip <IP_ADDRESS>    Use specific ip address instead of local broadcast [default:
                                 255.255.255.255]
    -p, --use-port <PORT>        Use specific port instead of default [default: 9]

```
## Config files
On linux, `~/.config/wakeup/config.toml` is used when looking for a config file.
