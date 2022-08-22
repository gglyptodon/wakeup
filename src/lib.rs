use clap::{builder::NonEmptyStringValueParser, Arg, ArgGroup, Command};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::net::UdpSocket;
use directories_next::{ProjectDirs};
use serde::Deserialize;

type WakeUpResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    machine_name: Option<String>,
    ip_address: Option<String>,
    port: Option<u16>,
    mac_address: Option<String>,
    debug: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct Machine {
    mac_addresses: Vec<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MachinePool{
    machines: Vec<Machine>
}

impl Machine {
    fn new(mac_addresses: Vec<String>, name: String) -> Self {
        Machine { mac_addresses, name }
    }
}
impl Display for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn get_args() -> WakeUpResult<Config> {
    let ip_addr_re: Regex = Regex::new("^(?:[0-9]{1,3}.){3}[0-9]{1,3}$").unwrap();
    let mac_addr_re: Regex = Regex::new("^([0-9A-Fa-f]{2}:){5}([0-9A-Fa-f]{2})$").unwrap();
    let matches = Command::new("wakeup")
        .about("Wake up a machine in the network with (a) magic (packet)")
        .arg(Arg::new("hostname")
                 .value_parser(NonEmptyStringValueParser::new())
                 .help("the name of the machine you want to wake up"),
                )
        .arg(Arg::new("mac_address")
                 .validator(|x|
                     if mac_addr_re.is_match(x){ Ok(()) }
                     else {
                        Err("Invalid format for mac address. Please use ':' as separator between hex digits.")
                     }
                 )
                .conflicts_with("hostname")
                .help("the mac address of the machine you want to wake up"),
        )
        .arg(
            Arg::new("debug")
                .long("--debug")
                .takes_value(false)
                .help("Print debug output to stdout"),
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

    let machine_name = matches.get_one::<String>("hostname").map(String::from);
    let mac_address = matches.get_one::<String>("mac_address").map(String::from);
    let ip_address = matches.get_one::<String>("use_ip").map(String::from);
    let port = matches.get_one::<u16>("port").copied();
    let debug = matches.is_present("debug");

    Ok(Config {
        machine_name,
        ip_address,
        port,
        mac_address,
        debug,
    })
}

pub fn run(config: Config) -> WakeUpResult<()> {
    if config.debug {
        println!("config: {:#?}", &config);
    }

    // named machine mode
    if let Some(name) = &config.machine_name {
        if read_config().is_err(){
            eprintln!("Could not read config file. You can still use a mac-address.")
        }
        let machines = read_config()?;
        if config.debug {
            for item in &machines {
                println!("debug: {:?}", item);
            }
        }
        if let Some(available) = machines.get(name) {
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
        let anon = Machine::new(vec![config.mac_address.unwrap()], "".to_string());
        println!("Trying to wake up host at < {} >", &anon.mac_addresses.get(0).unwrap());
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

fn read_config() -> WakeUpResult<HashMap<String, Machine>> {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "gglyptodon",  "wakeup") {
        let conf_dir = proj_dirs.config_dir();
        let config_name = "config.toml";
        let input = fs::read_to_string(conf_dir.join(config_name))?;
        //println!("input:{}", &input);
        let pool: MachinePool = toml::from_str(&input).unwrap();
        //println!("Pool {:?}", &pool);
        let mut machines: HashMap<String, Machine> = HashMap::new();
        for m in pool.machines.iter() {
            machines.insert(m.name.clone(), m.clone());
        }
        Ok(machines)
    }else{
        return Err(Box::new(ConfigFileNotFoundError))
    }


    //let config_path = "/etc/wakeup/wakeup.conf";
    //let input = File::open(config_path)?;
    //let buffered = BufReader::new(input);
    //for line in buffered.lines().flatten() {
    //    let tmp = line.split(',').collect::<Vec<&str>>();
    //    if tmp.len() != 2 {
    //        if conf.debug {
     //           eprintln!("debug: {:?} <- invalid line", tmp);
     //       }
     //       return Err(ConfigError.into());
    //    }
    //    let m = Machine::new(
    //        tmp.get(0).unwrap().to_string(),
    //        tmp.get(1).unwrap().to_string(),
     //   );
     //   if conf.debug {
     //       println!("debug: {:?}", m);
     //   }
     //   machines.insert(m.name.clone(), m);
   // }

    //Ok(machines)
}
/*
 The magic packet is a frame that is most often sent as a broadcast and that contains
 anywhere within its payload 6 bytes of all 255 (FF FF FF FF FF FF in hexadecimal),
 followed by sixteen repetitions of the target computer's 48-bit MAC address,
 for a total of 102 bytes.
*/
fn send_magic_packet(
    machine: &Machine,
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
    let magic_packet = craft_magic_packet(&machine.mac_addresses.get(0).unwrap())?;//todo
    let udp_socket = UdpSocket::bind("0.0.0.0:0")?;
    udp_socket.set_broadcast(true)?;
    udp_socket.send_to(&magic_packet, &destination)?;
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
    #[test]
    fn test_test() {
        assert_eq!(1, 1)
    }
}
