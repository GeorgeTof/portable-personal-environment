use std::process::{Command, Stdio};
use std::path::Path;
use crate::types::{DistroFamily, Task, TaskStatus};

pub fn remove_configuration(host_distribution: &DistroFamily) -> Result<String, String> {
    let tasks = vec![
        Task { 
            description: "Stop active PPE systemd mounts", 
            action: stop_systemd_mounts 
        },
        Task { 
            description: "Remove configuration files and scripts", 
            action: remove_files 
        },
        Task { 
            description: "Clean LXC virtual ethernet permissions", 
            action: clean_usernet 
        },
    ];

    println!("Starting host configuration removal...\n");
    let mut modified_configurations = false;

    for (i, task) in tasks.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, tasks.len(), task.description);

        match (task.action)(host_distribution) {
            Ok(TaskStatus::ActionCompleted) => {
                println!("{:>75}", "[ DONE ]\n");
                modified_configurations = true;
            }
            Ok(TaskStatus::JustChecked) => {
                println!("{:>75}", "[ ALREADY REMOVED ]\n");
            }
            Err(error_msg) => {
                println!("{:>75}", "[ FAILED ]\n");
                eprintln!("\nFatal Error: {}", error_msg);
                return Err(error_msg);
            }
        }
    }

    if modified_configurations {
        reload_daemons(host_distribution);
        Ok("Host configuration successfully removed!".to_string())
    } else {
        Ok("System checked! No PPE configurations were found on this host.".to_string())
    }
}

fn stop_systemd_mounts(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    let mut changed = false;
    let units = ["mnt-ppe.automount", "mnt-ppe.mount"];

    for unit in units {
        let is_active = Command::new("systemctl").arg("is-active").arg("--quiet").arg(unit)
            .status().map(|s| s.success()).unwrap_or(false);

        if is_active {
            let _ = Command::new("systemctl").arg("stop").arg(unit)
                .stdout(Stdio::null()).stderr(Stdio::null()).status();
            changed = true;
        }
    }
    
    if changed { Ok(TaskStatus::ActionCompleted) } else { Ok(TaskStatus::JustChecked) }
}

fn remove_files(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    let files_to_remove = vec![
        "/etc/udev/rules.d/99-lxc-gpu.rules",
        "/etc/udev/rules.d/99-ignore-ppe.rules",
        "/etc/systemd/system/mnt-ppe.mount",
        "/etc/systemd/system/mnt-ppe.automount",
        "/etc/sudoers.d/99-ppe-mount",
        "/usr/local/bin/ppe-launcher-stub",
        "/usr/share/applications/ppe-launcher.desktop",
    ];

    let mut changed = false;

    for file_path in files_to_remove {
        if Path::new(file_path).exists() {
            std::fs::remove_file(file_path).map_err(|e| format!("Failed to remove {}: {}", file_path, e))?;
            changed = true;
        }
    }

    if changed { Ok(TaskStatus::ActionCompleted) } else { Ok(TaskStatus::JustChecked) }
}

fn clean_usernet(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    let sudo_user = std::env::var("SUDO_USER").map_err(|_| "Please run with sudo!".to_string())?;
    let usernet_path = "/etc/lxc/lxc-usernet";
    let target_line = format!("{} veth lxcbr0 10", sudo_user);

    if !Path::new(usernet_path).exists() { return Ok(TaskStatus::JustChecked); }

    let contents = std::fs::read_to_string(usernet_path)
        .map_err(|e| format!("Failed to read {}: {}", usernet_path, e))?;

    if !contents.lines().any(|line| line.trim() == target_line) {
        return Ok(TaskStatus::JustChecked);
    }

    let new_contents: Vec<&str> = contents.lines()
        .filter(|line| line.trim() != target_line)
        .collect();

    let mut new_file_data = new_contents.join("\n");
    if !new_file_data.is_empty() { new_file_data.push('\n'); }

    std::fs::write(usernet_path, new_file_data)
        .map_err(|e| format!("Failed to rewrite {}: {}", usernet_path, e))?;

    Ok(TaskStatus::ActionCompleted)
}

fn reload_daemons(_distro: &DistroFamily) {
    let _ = Command::new("systemctl").arg("daemon-reload").status();
    let _ = Command::new("udevadm").arg("control").arg("--reload-rules").status();
    let _ = Command::new("update-desktop-database").arg("/usr/share/applications").status();
}