mod magic;

use clap::{Arg, Command};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{write, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};

type WakeUpResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    machine_name: String,
    debug: bool,
}

#[derive(Debug)]
struct Machine {
    ip_address: String,
    mac_address: String,
    name: String,
}

impl Machine {
    fn new(ip: String, mac: String, name: String) -> Self {
        Machine {
            ip_address: ip,
            mac_address: mac,
            name,
        }
    }
}
impl Display for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn get_args() -> WakeUpResult<Config> {
    let matches = Command::new("wakeup")
        .about("Wake up a machine in the network with (a) magic (packet)")
        .arg(
            Arg::new("machine")
                .allow_invalid_utf8(true)
                .required(true)
                .help("the name of the machine you want to wake up"),
        )
        .arg(
            Arg::new("debug")
                .long("--debug")
                .takes_value(false)
                .help("Print debug output to stdout"),
        )
        .get_matches();
    let machine_name = matches.value_of_lossy("machine").unwrap().to_string();
    let debug = matches.is_present("debug");
    Ok(Config {
        machine_name,
        debug,
    })
}

pub fn run(config: Config) -> WakeUpResult<()> {
    if config.debug {
        println!("config: {:#?}", config);
    }
    let machines = read_config(&config)?;
    if config.debug {
        for item in &machines {
            println!("debug: {:?}", item);
        }
    }
    if let Some(available) = machines.get(&config.machine_name) {
        println!("Waking up < {} >", available);
        println!("Check back in a few minutes");
        send_magic_packet(available)?
    } else {
        eprintln!("Name provided: {}", &config.machine_name);
        return Err(UnknownHostError.into());
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

fn read_config(conf: &Config) -> WakeUpResult<HashMap<String, Machine>> {
    //let mut machines: Vec<Machine> = Vec::new();
    let mut machines: HashMap<String, Machine> = HashMap::new();
    let config_path = "/etc/wakeup/wakeup.conf";
    let input = File::open(config_path)?;
    let buffered = BufReader::new(input);
    for line in buffered.lines() {
        if let Ok(l) = line {
            let tmp = l.split(',').collect::<Vec<&str>>();
            if tmp.len() != 3 {
                if conf.debug {
                    eprintln!("debug: {:?} <- invalid line", tmp);
                }
                return Err(ConfigError.into());
            }
            let m = Machine::new(
                tmp.get(0).unwrap().to_string(),
                tmp.get(1).unwrap().to_string(),
                tmp.get(2).unwrap().to_string(),
            );
            if conf.debug {
                println!("debug: {:?}", m);
            }
            //machines.push(m);
            machines.insert(m.name.clone(), m);
        }
    }

    Ok(machines)
}
/*The magic packet is a frame that is most often sent as a broadcast and that contains
 anywhere within its payload 6 bytes of all 255 (FF FF FF FF FF FF in hexadecimal),
  followed by sixteen repetitions of the target computer's 48-bit MAC address,
  for a total of 102 bytes.
 */
fn send_magic_packet(machine: &Machine)-> WakeUpResult<()>{
    let mac_as_bytes = convert_mac(&machine.mac_address)?;
    println!("{:?}", mac_as_bytes);
    Ok(())
}


fn convert_mac(mac: &String) -> Result<Vec<u8>, String> {
    let splits = mac.split(":");
    let result = splits.into_iter().filter_map(|x|hex::decode(x).ok()).flatten().collect::<Vec<u8>>();
    return if result.len() == 6 {
        Ok(result)
    } else { Err(format!("invalid Mac: {}-> {:?}",&mac,&result)).into() }

}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
        assert_eq!(1, 1)
    }
}
