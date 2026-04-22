use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::types::{DistroFamily};


pub struct Task {
    description: &'static str,
    action: fn(&DistroFamily) -> Result<TaskStatus, String>, 
}

pub enum TaskStatus {
    Configured,
    JustChecked,
}

pub fn configure(host_distribution: &DistroFamily) -> Result<String, String> {
    let tasks = vec![
        Task { 
            description: "Install required packages",
            action: install_prerequisites 
        },
        Task { 
            description: "TODO", 
            action: allow_veth_connections 
        },
        Task { 
            description: "TODO", 
            action: create_mount_location 
        },
        Task { 
            description: "TODO", 
            action: access_gpu_devices 
        },
        Task { 
            description: "TODO", 
            action: auto_mount 
        },
        Task { 
            description: "TODO", 
            action: create_intermediary_script 
        },
        Task { 
            description: "TODO", 
            action: link_desktop_shrtcut 
        },
    ];

    println!("Starting host configuration...\n");

    for (i, task) in tasks.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, tasks.len(), task.description);

        match (task.action)(host_distribution) {
            Ok(TaskStatus::Configured) => {
                println!("{:>75}", "[ DONE ]\n"); // todo: maybe add emojis / visual clue
            }
            Ok(TaskStatus::JustChecked) => {
                println!("{:>75}", "[ ALREADY CONFIGURED ]\n");
            }
            Err(error_msg) => {
                println!("{:>75}", "[ FAILED ]\n");
                eprintln!("\nFatal Error: {}", error_msg);
                eprintln!("Aborting configuration.");
                return Err(error_msg);
            }
        }
    }

    Ok("Host configuration completed successfully!".to_string())
}


fn install_prerequisites(distro: &DistroFamily) -> Result<TaskStatus, String> { // optional todo : have some sort of real time feedback during the long instalation
    let required_packages = vec!["lxc", "lxc-utils", "lxc-templates", "uidmap", "acl"];
    let mut installed = Vec::new();
    let mut missing = Vec::new();

    match distro {
        DistroFamily::Debian => check_debian_prerequisites(&required_packages, &mut installed, &mut missing)?,
        _ => return Err("Distribution family packet manager not supported".to_string()),
    }

    if !installed.is_empty() {
        println!("\n  -> Already installed: {}", installed.join(", "));
    }

    if missing.is_empty() {
        return Ok(TaskStatus::JustChecked);
    }

    print!("\n  -> Starting installation of: {} ...", missing.join(", "));
    io::stdout().flush().map_err(|e| e.to_string())?;

    match distro {
        DistroFamily::Debian => install_debian_prerequisites(&missing)?,
        _ => return Err("Distribution family packet manager not supported".to_string()),
    }

    Ok(TaskStatus::Configured)
}

fn check_debian_prerequisites<'a>(packages: &[&'a str], installed: &mut Vec<&'a str>, missing: &mut Vec<&'a str>) -> Result<(), String> {

    for &pkg in packages {
        let status = Command::new("dpkg")
            .arg("-s")
            .arg(pkg)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("Failed to execute dpkg: {}", e))?;

        if status.success() {
            installed.push(pkg);
        } else {
            missing.push(pkg);
        }
    }    

    Ok(())
}

fn install_debian_prerequisites(missing: &[&str]) -> Result<(), String> {
    // update apt cache first silently
    let _ = Command::new("apt-get")
        .arg("update")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let mut install_cmd = Command::new("apt-get");
    install_cmd
        .env("DEBIAN_FRONTEND", "noninteractive")
        .arg("install")
        .arg("-y");

    for pkg in missing {
        install_cmd.arg(pkg);
    }

    // can check real-time status in another terminal with: tail -f /var/log/dpkg.log
    let install_status = install_cmd
        .stdout(Stdio::null()) 
        .stderr(Stdio::piped()) // suppressed, can capture stderr to debug if needed
        .status()
        .map_err(|e| format!("Failed to execute apt-get install: {}", e))?;

    if install_status.success() {
        println!("\n  -> Installation successful!");
        Ok(())
    } else {
        Err(format!("Failed to install packages via apt-get. Ensure you have internet connectivity."))
    }
}

fn allow_veth_connections(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    Err("Could not connect to server".to_string())
}

fn create_mount_location(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    Err("NOT IMPLEMENTED.".to_string())
}

fn access_gpu_devices(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    Err("NOT IMPLEMENTED.".to_string())
}

fn auto_mount(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    Err("NOT IMPLEMENTED.".to_string())
}

fn create_intermediary_script(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    Err("NOT IMPLEMENTED.".to_string())
}

fn link_desktop_shrtcut(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    Err("NOT IMPLEMENTED.".to_string())
}