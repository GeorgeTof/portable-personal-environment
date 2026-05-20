use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::configure::{configure_acls, configure_wayland_access};
use crate::utils::{run_lxc_shell_cmd, get_blacklisted_users};
use crate::session::{SessionState, initial_session_config};

#[tauri::command]
pub fn check_host_config() -> (bool, Option<String>) {
    let user = match env::var("USER") {
        Ok(u) => u,
        Err(_) => return (false, Some("Could not determine current user.".to_string())),
    };

    for bin in &["lxc-start", "newuidmap"] {
        if !command_exists(bin) {
            return (false, Some(format!("Required tool '{}' is not installed on this host.", bin)));
        }
    }

    if !file_contains_line_starting_with("/etc/subuid", &user) {
        return (false, Some(format!("No subUID allocation found for user '{}'.", user)));
    }
    if !file_contains_line_starting_with("/etc/subgid", &user) {
        return (false, Some(format!("No subGID allocation found for user '{}'.", user)));
    }

    let usernet_expected = format!("{} veth lxcbr0", user);
    if !file_contains_string("/etc/lxc/lxc-usernet", &usernet_expected) {
        return (false, Some(format!("Network permissions for '{}' not found in /etc/lxc/lxc-usernet.", user)));
    }

    // #todo pls-double-check - for discrete gpu
    let dri_path = "/dev/dri";
    if Path::new(dri_path).exists() {
        if let Ok(entries) = fs::read_dir(dri_path) {
            let mut found_devices = false;
            let mut missing_acl = false;

            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                
                // only check actual DRM device nodes
                if file_name.starts_with("card") || file_name.starts_with("render") {
                    found_devices = true;
                    if let Some(path_str) = path.to_str() {
                        if !has_acl(path_str, "100000", "rw-") {
                            missing_acl = true;
                            break;
                        }
                    }
                }
            }

            if found_devices && missing_acl {
                return (false, Some("Container root (100000) lacks read/write ACLs for one or more GPU devices inside /dev/dri/.".to_string()));
            }
        }
    }
    configure_wayland_access()
}

#[tauri::command]
pub fn check_drive_configuration() -> (bool, Option<String>) {
    let mount_dir = "/mnt/ppe";
    let lxc_data_dir = "/mnt/ppe/lxc-data";
    let container_dir = "/mnt/ppe/lxc-data/personal-container";
    let config_file = format!("{}/config", container_dir);
    let rootfs_dir = format!("{}/rootfs", container_dir);

    if Path::new(mount_dir).exists() && !has_acl(mount_dir, "100000", "--x") {
        if let Err(e) = configure_acls(mount_dir) {
            return (false, Some(format!("Auto-configuration failed for mount root ACLs: {}", e)));
        }
    }

    if !Path::new(container_dir).is_dir() {
        return (false, None);
    }

    if !Path::new(&config_file).is_file() {
        return (false, Some(format!("Container configuration file missing: {}", config_file)));
    }

    if !Path::new(&rootfs_dir).is_dir() {
        return (false, Some(format!("Container root filesystem missing: {}", rootfs_dir)));
    }

    if !has_acl(lxc_data_dir, "100000", "--x") {
        if let Err(e) = configure_acls(lxc_data_dir) {
            return (false, Some(format!("Auto-configuration failed for lxc-data ACLs: {}", e)));
        }
    }

    (true, None)
}

#[tauri::command]
pub fn check_container_integrity() -> (bool, Option<String>) {
    let rootfs_dir = "/mnt/ppe/lxc-data/personal-container/rootfs";

    let essential_files = [
        "sbin/init",   // primary boot executable (often a symlink to systemd)
        "bin/sh",      // standard shell required for basic scripts
        "etc/passwd",  // fundamental user database
    ];

    for file in essential_files.iter() {
        let full_path = format!("{}/{}", rootfs_dir, file);
        
        // Note: .exists() automatically resolves and follows symlinks, 
        if !Path::new(&full_path).exists() {
            return (false, Some(format!(
                "Container integrity compromised: Essential file '{}' is missing.", 
                full_path
            )));
        }
    }

    (true, None)
}

#[tauri::command]
pub fn check_container_runtime_integrity() -> (bool, Option<String>) {
    // TODO
    (true, None)
}

#[tauri::command]
pub fn check_configured_for_personal_use(state: tauri::State<'_, SessionState>) -> (bool, Option<String>) {
let check_cmd = "getent passwd | awk -F: '$3 >= 1000 && $3 < 65534 {print $1}'";
    let blacklist = get_blacklisted_users();

    if let Ok(existing_users) = run_lxc_shell_cmd(check_cmd) {
        for user in existing_users.lines() {
            let u = user.trim();
            if u.is_empty() { continue; }

            if !blacklist.contains(&u.to_string()) {
                let (dbus_ok, dbus_err) = initial_session_config(&u.to_string());
                if !dbus_ok {
                    println!("Error starting runtime D-Bus: {:?}", dbus_err);
                    let _ = std::process::Command::new("notify-send")
                        .arg("PPE Launcher Warning")
                        .arg("Failed to initialize the D-Bus.")
                        .status();
                }

                // cache the found valid user
                let mut lock = state.current_user.lock().unwrap();
                *lock = Some(u.to_string());
                return (true, None);
            }
        }
    }

    (false, None)
}


pub fn has_acl(path: &str, uid: &str, permissions_substring: &str) -> bool {
    // getfacl -n outputs numeric UIDs, making it safe to parse
    match Command::new("getfacl").arg("-n").arg(path).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let expected_rule = format!("user:{}:{}", uid, permissions_substring);
            stdout.contains(&expected_rule)
        }
        _ => false,
    }
}


fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn file_contains_line_starting_with(path: &str, prefix: &str) -> bool {
    if let Ok(contents) = fs::read_to_string(path) {
        contents.lines().any(|line| line.starts_with(prefix))
    } else {
        false
    }
}

fn file_contains_string(path: &str, search: &str) -> bool {
    if let Ok(contents) = fs::read_to_string(path) {
        contents.contains(search)
    } else {
        false
    }
}