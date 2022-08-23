use assert_cmd::Command;
use predicates::prelude::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const PRG: &str = "wakeup";

fn run(args: &[&str], expected: &str) -> TestResult {
    let cmd = Command::cargo_bin(PRG)?.args(args).assert().success();
    let out = cmd.get_output();
    let stdout = String::from_utf8(out.stdout.clone())?;
    let stderr = String::from_utf8(out.stderr.clone())?;
    let mut lines: Vec<&str> = stdout.split("\n").filter(|s| !s.is_empty()).collect();
    println!("{:?} - {:?}", lines, expected);
    //assert_eq!(lines, expected);
    Ok(())
}

fn run_fail_contains(args: &[&str], expected: &str) -> TestResult {
    Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

//cli
#[test]
fn test_dies_called_with_hostname_and_mac() -> TestResult {
    let expected = "The argument '<hostname>' cannot be used with '--mac <MAC_ADDRESS>'";
    run_fail_contains(&["test", "-m", "aa:bb:cc:dd:ee:ff"], expected)
}
#[test]
fn test_dies_called_with_invalid_mac() -> TestResult {
    let expected = "Invalid format for mac address. Please use ':' as separator";
    run_fail_contains(&["-m", "aa:bb:cc:dd:ee:ff:gg"], expected)
}
#[test]
fn test_dies_called_with_invalid_mac_invalid_digits() -> TestResult {
    let expected = "Invalid format for mac address. Please use ':' as separator";
    run_fail_contains(&["-m", "aa:bb:cc:dd:gg:hh"], expected)
}

#[test]
fn test_dies_called_with_invalid_ip() -> TestResult {
    let expected = "Invalid format for ip address";
    run_fail_contains(&["-m", "aa:bb:cc:dd:ee:ff", "-i", "127.0.0.a"], expected)
}

#[test]
fn test_dies_called_with_invalid_port()->TestResult {
     let expected = "invalid digit found in string";
    run_fail_contains(&["-m", "aa:bb:cc:dd:ee:ff", "-i", "127.0.0.1", "-p", "a"], expected)
}

//config
fn test_dies_no_config_called_with_hostname() {}
fn test_ok_no_config_called_with_mac() {}
fn test_dies_no_mac_in_config_called_with_hostname() {}
fn test_dies_invalid_mac_in_config_called_with_hostname() {}

// os and config
#[cfg(linux)]
fn test_finds_config_linux() {}
#[cfg(windows)]
fn test_finds_config_windows() {}
#[cfg(macos)]
fn test_finds_config_macos() {}
