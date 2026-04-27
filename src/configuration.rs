use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::os::unix::fs::MetadataExt; // heavy import

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
            description: "Allow user to create virtual ethernet connections", 
            action: allow_veth_connections 
        },
        Task { 
            description: "Create mount point for the external drive", 
            action: create_mount_location 
        },
        Task { 
            description: "Create custom udev rule for granting GPU access",
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
                println!("{:>75}", "[ DONE ]\n"); // todo: maybe add emojis / visual cue
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
    let sudo_user = std::env::var("SUDO_USER").map_err(|_| {
        "Could not determine original user.".to_string()
    })?;

    let usernet_path = "/etc/lxc/lxc-usernet";
    let expected_line = format!("{} veth lxcbr0 10", sudo_user);

    if let Ok(contents) = std::fs::read_to_string(usernet_path) {
        if contents.lines().any(|line| line.trim() == expected_line) {
            return Ok(TaskStatus::JustChecked);
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(usernet_path)
        .map_err(|e| format!("Failed to open {}: {}", usernet_path, e))?;

    writeln!(file, "{}", expected_line)
        .map_err(|e| format!("Failed to write to {}: {}", usernet_path, e))?;

    Ok(TaskStatus::Configured)
}

fn create_mount_location(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    let mount_path = "/mnt/ppe";
    
    let sudo_user = std::env::var("SUDO_USER").map_err(|_| {
        "Could not determine original user.".to_string()
    })?;

    // import increases executable size by 4MB (0.5 to 4.5 MB) -> todo: consider if worth keeping it
    // all following operations are idempotent anyway, may just check if the directory exists
    if std::path::Path::new(mount_path).exists() {
        if let Ok(metadata) = std::fs::metadata(mount_path) {
            
            // get the numeric UID and GID of the SUDO_USER
            let uid_output = Command::new("id").arg("-u").arg(&sudo_user).output();
            let gid_output = Command::new("id").arg("-g").arg(&sudo_user).output();

            if let (Ok(uid_res), Ok(gid_res)) = (uid_output, gid_output) {
                let expected_uid = String::from_utf8_lossy(&uid_res.stdout).trim().parse::<u32>().unwrap_or(0);
                let expected_gid = String::from_utf8_lossy(&gid_res.stdout).trim().parse::<u32>().unwrap_or(0);

                let is_correct_owner = metadata.uid() == expected_uid && metadata.gid() == expected_gid;
                
                let has_others_exec = (metadata.mode() & 0b000_000_001) != 0;

                if is_correct_owner && has_others_exec {
                    return Ok(TaskStatus::JustChecked);
                }
            }
        }
    }

    print!("\n  -> Creating the mount location: {} ...", mount_path);
    std::fs::create_dir_all(mount_path)
        .map_err(|e| format!("Failed to create directory {}: {}", mount_path, e))?;

    print!("\n  -> Making current user its owner");
    // (User:Group)
    let chown_status = Command::new("chown")
        .arg(format!("{}:{}", sudo_user, sudo_user)) 
        .arg(mount_path)
        .status()
        .map_err(|e| format!("Failed to execute chown: {}", e))?;

    if !chown_status.success() {
        return Err(format!("Failed to set ownership for {}", mount_path));
    }

    print!("\n  -> Allowing traversal for all users \n");
    let chmod_status = Command::new("chmod")
        .arg("o+x")
        .arg(mount_path)
        .status()
        .map_err(|e| format!("Failed to execute chmod: {}", e))?;

    if !chmod_status.success() {
        return Err(format!("Failed to set traversal permissions for {}", mount_path));
    }

    Ok(TaskStatus::Configured)
}

fn access_gpu_devices(_distro: &DistroFamily) -> Result<TaskStatus, String> {
    let rule_path = "/etc/udev/rules.d/99-lxc-gpu.rules";
    let rule_content = "ACTION==\"add|change\", SUBSYSTEM==\"drm\", RUN+=\"/usr/bin/setfacl -m u:100000:rw $env{DEVNAME}\"\n";

    if let Ok(existing_content) = std::fs::read_to_string(rule_path) {
        if existing_content == rule_content {
            return Ok(TaskStatus::JustChecked);
        }
    }

    std::fs::write(rule_path, rule_content)
        .map_err(|e| format!("Failed to write udev rule to {}: {}", rule_path, e))?;

    // reload the udev rules - for immediate effect
    let reload_status = Command::new("udevadm")
        .arg("control")
        .arg("--reload-rules")
        .status()
        .map_err(|e| format!("Failed to execute udevadm control: {}", e))?;

    if !reload_status.success() {
        return Err("Failed to reload udev rules".to_string());
    }

    // trigger the new rule - to apply immediately to existing GPU devices
    let trigger_status = Command::new("udevadm")
        .arg("trigger")
        .arg("--subsystem-match=drm")
        .status()
        .map_err(|e| format!("Failed to execute udevadm trigger: {}", e))?;

    if !trigger_status.success() {
        return Err("Failed to trigger udev rules".to_string());
    }

    Ok(TaskStatus::Configured)
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