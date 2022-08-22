use clap::{builder::NonEmptyStringValueParser, Arg, ArgGroup, Command};
use directories_next::ProjectDirs;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::net::UdpSocket;

type WakeUpResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    host_name: Option<String>,
    ip_address: Option<String>,
    port: Option<u16>,
    mac_address: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Host {
    mac_addresses: Vec<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct HostPool {
    hosts: Vec<Host>,
}

impl Host {
    fn new(mac_addresses: Vec<String>, name: String) -> Self {
        Host {
            mac_addresses,
            name,
        }
    }
}
impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn get_args() -> WakeUpResult<Config> {
    let ip_addr_re: Regex = Regex::new("^(?:[0-9]{1,3}.){3}[0-9]{1,3}$").unwrap();
    let mac_addr_re: Regex = Regex::new("^([0-9A-Fa-f]{2}:){5}([0-9A-Fa-f]{2})$").unwrap();
    let matches = Command::new("wakeup")
        .about("Wake up a host in the network with (a) magic (packet)")
        .arg(Arg::new("hostname")
                 .value_parser(NonEmptyStringValueParser::new())
                 .help("the name of the host you want to wake up"),
                )
        .arg(Arg::new("mac_address")
                 .validator(|x|
                     if mac_addr_re.is_match(x){ Ok(()) }
                     else {
                        Err("Invalid format for mac address. Please use ':' as separator between hex digits.")
                     }
                 )
                .conflicts_with("hostname")
                .help("the mac address of the host you want to wake up"),
        )
        .arg(Arg::new("use_ip")
            .long("use-ip")
            .short('i')
            .value_name("IP_ADDRESS")
            .takes_value(true)
            .value_parser(NonEmptyStringValueParser::new())
            .help("Use specific ip address instead of local broadcast")
            .required(false)
            .default_value("255.255.255.255")
            .validator(|x|if ip_addr_re.is_match(x){Ok(())}else{Err("Invalid format for ip address")})
        )
        .arg(Arg::new("port")
            .long("use-port")
            .short('p')
            .value_name("PORT").takes_value(true)
            .value_parser(clap::value_parser!(u16).range(0..))
            .help("Use specific port instead of default")
            .required(false)
            .default_value("9")
        )
        .group(ArgGroup::with_name("target")
            .args(&["hostname", "mac_address"])
            .multiple(false)
            .required(true)
        )
        .get_matches();

    let host_name = matches.get_one::<String>("hostname").map(String::from);
    let mac_address = matches.get_one::<String>("mac_address").map(String::from);
    let ip_address = matches.get_one::<String>("use_ip").map(String::from);
    let port = matches.get_one::<u16>("port").copied();

    Ok(Config {
        host_name,
        ip_address,
        port,
        mac_address,
    })
}

pub fn run(config: Config) -> WakeUpResult<()> {
    // named mode
    if let Some(name) = &config.host_name {
        if read_config().is_err() {
            eprintln!("Could not read config file. You can still use a mac-address.")
        }
        let hosts = read_config()?;
        if let Some(available) = hosts.get(name) {
            println!("Trying to wake up < {} >", available);
            send_magic_packet(available, &config.ip_address, &config.port)?;
            println!("Magic packet sent. Check back in a few minutes.");
        } else {
            eprintln!("Name provided: {}", name);
            return Err(UnknownHostError.into());
        }
    }
    // mac address mode
    else {
        let anon = Host::new(vec![config.mac_address.unwrap()], "".to_string());
        println!(
            "Trying to wake up host at < {} >",
            &anon.mac_addresses.get(0).unwrap()
        );
        send_magic_packet(&anon, &config.ip_address, &config.port)?;
        println!("Magic packet sent. Check back in a few minutes.");
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ConfigError;
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Invalid config.")
    }
}

impl Error for ConfigError {}

#[derive(Debug, Clone)]
pub struct UnknownHostError;
impl Display for UnknownHostError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Not a known host.")
    }
}
impl Error for UnknownHostError {}

#[derive(Debug, Clone)]
pub struct ConfigFileNotFoundError;
impl Display for ConfigFileNotFoundError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Could not find config file.")
    }
}
impl Error for ConfigFileNotFoundError {}

#[derive(Debug, Clone)]
pub struct MagicError;
impl Display for MagicError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Something went wrong while crafting the magic package.")
    }
}
impl Error for MagicError {}

fn read_config() -> WakeUpResult<HashMap<String, Host>> {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "gglyptodon", "wakeup") {
        let conf_dir = proj_dirs.config_dir();
        let config_name = "config.toml";
        let input = fs::read_to_string(conf_dir.join(config_name))?;
        let pool: HostPool = toml::from_str(&input).unwrap();
        let mut hosts: HashMap<String, Host> = HashMap::new();
        for m in pool.hosts.iter() {
            hosts.insert(m.name.clone(), m.clone());
        }
        Ok(hosts)
    } else {
        Err(Box::new(ConfigFileNotFoundError))
    }
}

/*
 The magic packet is a frame that is most often sent as a broadcast and that contains
 anywhere within its payload 6 bytes of all 255 (FF FF FF FF FF FF in hexadecimal),
 followed by sixteen repetitions of the target computer's 48-bit MAC address,
 for a total of 102 bytes.
*/
fn send_magic_packet(
    host: &Host,
    address: &Option<String>,
    port: &Option<u16>,
) -> WakeUpResult<()> {
    let destination_address = match address {
        Some(val) => val.clone(),
        None => String::from("255.255.255.255"),
    };
    let destination_port = match port {
        Some(val) => val.to_string(),
        None => 9.to_string(),
    };
    let destination = format!("{}:{}", destination_address, destination_port);
    let mut magic_packets: Vec<Vec<u8>> = Vec::new();
    for mac_address in &host.mac_addresses {
        magic_packets.push(craft_magic_packet(mac_address)?);
    }
    let udp_socket = UdpSocket::bind("0.0.0.0:0")?;
    udp_socket.set_broadcast(true)?;
    for magic_packet in magic_packets {
        udp_socket.send_to(&magic_packet, &destination)?;
    }
    Ok(())
}

fn craft_magic_packet(mac_address: &String) -> WakeUpResult<Vec<u8>> {
    let mac_as_bytes = convert_mac(mac_address)?;
    let mut magic: Vec<u8> = vec![0xff; 6];
    let mut reps: Vec<u8> = Vec::new();
    for _ in 0..16 {
        reps.extend(&mac_as_bytes)
    }
    magic.extend(&reps);
    if magic.len() != 102 {
        return Err(MagicError.into());
    }
    Ok(magic)
}

fn convert_mac(mac: &String) -> Result<Vec<u8>, String> {
    let splits = mac.split(':');
    let result = splits
        .into_iter()
        .filter_map(|x| hex::decode(x).ok())
        .flatten()
        .collect::<Vec<u8>>();
    return if result.len() == 6 {
        Ok(result)
    } else {
        Err(format!("Invalid MAC: {}-> {:?}", &mac, &result))
    };
}

#[cfg(test)]
mod tests {
    use crate::{convert_mac, craft_magic_packet, Host, HostPool};

    #[test]
    fn test_test() {
        assert_eq!(1, 1)
    }

    #[test]
    fn test_valid_mac() {
        let mac = "aa:bb:cc:dd:ee:ff".to_string();
        let expected: Vec<u8> = vec![170, 187, 204, 221, 238, 255];
        let result = convert_mac(&mac);
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    #[should_panic]
    fn test_invalid_mac_short() {
        let mac = "bb:cc:dd:ee:ff".to_string();
        let _result = convert_mac(&mac).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_mac() {
        let mac = "gg:bb:cc:dd:ee:ff".to_string();
        let _result = convert_mac(&mac).unwrap();
    }

    #[test]
    fn test_valid_magic_len() {
        let mac = "aa:bb:cc:dd:ee:ff".to_string();
        let expected = 102;
        let result = craft_magic_packet(&mac);
        assert_eq!(expected, result.unwrap().len());
    }

    #[test]
    fn test_valid_magic() {
        let mac = "aa:bb:cc:dd:ee:ff".to_string();
        let expected = vec![
            255, 255, 255, 255, 255, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221, 238,
            255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221,
            238, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204,
            221, 238, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221, 238, 255, 170, 187,
            204, 221, 238, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221, 238, 255, 170,
            187, 204, 221, 238, 255, 170, 187, 204, 221, 238, 255, 170, 187, 204, 221, 238, 255,
        ];

        let result = craft_magic_packet(&mac);
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_parse_toml() {
        let mytoml = r#" hosts = [{ name= "my_host", mac_addresses = ["aa:bb:cc:dd:ee:ff"] },
        { name = "my_other_host", mac_addresses = ["00:11:22:33:ff:ff", "00:11:22:33:44:ff"] },]"#;
        let result = toml::from_str::<HostPool>(mytoml).unwrap();
        let expected = HostPool {
            hosts: vec![
                Host::new(vec!["aa:bb:cc:dd:ee:ff".to_string()], "my_host".to_string()),
                Host::new(
                    vec![
                        "00:11:22:33:ff:ff".to_string(),
                        "00:11:22:33:44:ff".to_string(),
                    ],
                    "my_other_host".to_string(),
                ),
            ],
        };
        assert_eq!(
            expected.hosts.get(0).unwrap().name,
            result.hosts.get(0).unwrap().name
        );
        assert_eq!(
            expected.hosts.get(1).unwrap().name,
            result.hosts.get(1).unwrap().name
        );
        assert_eq!(
            expected.hosts.get(0).unwrap().mac_addresses.get(0).unwrap(),
            result.hosts.get(0).unwrap().mac_addresses.get(0).unwrap()
        );
        assert_eq!(
            expected.hosts.get(1).unwrap().mac_addresses.get(0).unwrap(),
            result.hosts.get(1).unwrap().mac_addresses.get(0).unwrap()
        );
        assert_eq!(
            expected.hosts.get(1).unwrap().mac_addresses.get(1).unwrap(),
            result.hosts.get(1).unwrap().mac_addresses.get(1).unwrap()
        );
    }
}
