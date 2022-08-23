use assert_cmd::Command;
use predicates::prelude::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const PRG: &str = "wakeup";

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
fn test_dies_called_with_invalid_port() -> TestResult {
    let expected = "invalid digit found in string";
    run_fail_contains(
        &["-m", "aa:bb:cc:dd:ee:ff", "-i", "127.0.0.1", "-p", "a"],
        expected,
    )
}
