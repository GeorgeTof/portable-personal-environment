use std::fs;
use std::env;
use std::io::{self, Write};
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
    
    let is_help = env::args().any(|arg| arg == "--help") || env::args().any(|arg| arg == "-h");
    if is_help {
        println!("Simply run the script as sudo to configure the host machine or check its configuration");
        println!("     --remove   Remove the PPE configuration from the host");
        println!(" -h, --help     Show this help menu");
        return;
    }

    
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

    let target_user = get_user_to_configure();

    if is_removal {
        match removal::remove_configuration(&host_distribution, &target_user) {
            Ok(result) => println!("{result}"),
            Err(e) => eprintln!("Failed removal due to error: {e}"),
        }
    }
    else {
        match configuration::configure(&host_distribution, &target_user) {
            Ok(result) => {
                println!("{result}");
                println!("Please close the terminal window and remove the external drive.");
            },
            Err(e) => eprintln!("Failed configuration due to error: {e}"),
        }
    }
}

fn get_user_to_configure() -> String {
    let passwd_contents = fs::read_to_string("/etc/passwd").unwrap_or_default();
    let mut valid_users = Vec::new();

    for line in passwd_contents.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            if let Ok(uid) = parts[2].parse::<u32>() {
                if uid >= 1000 && uid < 65534 { // standard users start at 1000, 65534 is nobody
                    valid_users.push(parts[0].to_string());
                }
            }
        }
    }

    if valid_users.is_empty() {
        eprintln!("No standard users found on this system.");
        std::process::exit(1);
    }

    loop {
        println!("\nPlease select the user to configure the environment for:");
        for (i, user) in valid_users.iter().enumerate() {
            println!("  {}) {}", i + 1, user);
        }
        print!("\nEnter index or username: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Ok(index) = input.parse::<usize>() {
            if index > 0 && index <= valid_users.len() {
                return valid_users[index - 1].clone();
            }
        } else if valid_users.contains(&input.to_string()) {
            return input.to_string();
        }

        println!("Invalid selection. Please try again.");
    }
}