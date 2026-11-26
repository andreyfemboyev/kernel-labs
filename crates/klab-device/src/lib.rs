use klab_core::{Result, invalid, output};
use std::process::Command;

pub fn list(fastboot: bool) -> Result<String> {
    if fastboot {
        output(Command::new("fastboot").args(["devices", "-l"]))
    } else {
        output(Command::new("adb").args(["devices", "-l"]))
    }
}

fn adb(serial: &str) -> Result<Command> {
    if serial.is_empty() || serial.starts_with('-') || serial.chars().any(char::is_whitespace) {
        return Err(invalid("explicit device serial required"));
    }
    let mut command = Command::new("adb");
    command.args(["-s", serial, "shell"]);
    Ok(command)
}

pub fn kernel(serial: &str) -> Result<String> {
    output(adb(serial)?.args(["uname", "-a"]))
}
pub fn log(serial: &str) -> Result<String> {
    output(adb(serial)?.arg("dmesg"))
}
