use std::fs;
use std::env;
use crate::types::{DistroFamily};

mod configuration;
mod removal;
mod types;
mod templates;

fn initial_checks() -> (bool, DistroFamily){
    // unsafe is required due to making a direct C system call
    let is_root = unsafe { libc::geteuid() } == 0;

    let mut id = String::new();
    let mut id_like = String::new();

    if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
        for line in os_release.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                id = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
                id_like = val.trim_matches('"').to_string();
            }
        }
    }

    let distribution_family = format!("{} {}", id, id_like);

    (is_root, DistroFamily::from_os_string(&distribution_family))
}

fn main() {
    println!("Welcome to the host configuration for the Portable Personal Environment!\n\n");

    let (is_root, host_distribution) = initial_checks();

    println!("The detected distribution is based on {:?}", host_distribution);

    if !is_root {
        eprintln!("\n[WARNING] This configuration tool requires root privileges!\n");
        return;
    }
    if !host_distribution.is_supported() {
        eprintln!("\n[WARNING] The host distribution family is not currently supported!\n");
        return;
    }

    let is_removal = env::args().any(|arg| arg == "--remove");

    if is_removal {
        match removal::remove_configuration(&host_distribution) {
            Ok(result) => println!("{result}"),
            Err(e) => eprintln!("Failed removal due to error: {e}"),
        }
    }
    else {
        match configuration::configure(&host_distribution) {
            Ok(result) => {
                println!("{result}");
                println!("Please close the terminal window and remove the external drive.");
            },
            Err(e) => eprintln!("Failed configuration due to error: {e}"),
        }
    }
}
