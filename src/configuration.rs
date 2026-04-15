use std::io::{self, Write};

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
            description: "TODO",
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
        print!("[{}/{}] {:<35} ", i + 1, tasks.len(), task.description); 
        io::stdout().flush().unwrap();

        match (task.action)(host_distribution) {
            Ok(TaskStatus::Configured) => {
                println!("[ ✅ DONE ]");
            }
            Ok(TaskStatus::JustChecked) => {
                println!("[ ⏭️ ALREADY CONFIGURED ]");
            }
            Err(error_msg) => {
                println!("[ ❌ FAILED ]");
                eprintln!("\nFatal Error: {}", error_msg);
                eprintln!("Aborting configuration.");
                return Err(error_msg);
            }
        }
    }

    Ok("Host configuration completed successfully!".to_string())
}


fn install_prerequisites(distro: &DistroFamily) -> Result<TaskStatus, String> {
    Ok(TaskStatus::Configured) 
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