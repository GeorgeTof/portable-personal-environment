use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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

    // #pls-double-check - for discrete gpu
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

pub fn configure_wayland_access() -> (bool, Option<String>) {
    let runtime_dir = match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            // fallback to find the runtime directory
            match Command::new("id").arg("-u").output() {
                Ok(out) if out.status.success() => {
                    let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    format!("/run/user/{}", uid)
                }
                _ => return (false, Some("Failed to determine current user ID (id -u failed).".to_string())),
            }
        }
    };
    let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());
    let wayland_socket = format!("{}/{}", runtime_dir, wayland_display);

    if !Path::new(&wayland_socket).exists() {
        // (maybe add X11 logic as fallback)
        return (false, Some(format!("Wayland socket not found at {}", wayland_socket)));
    }

    // grant container root traversal access to the runtime directory (if not already set)
    if !has_acl(&runtime_dir, "100000", "--x") {
        let status_dir = Command::new("setfacl")
            .arg("-m")
            .arg("u:100000:x")
            .arg(&runtime_dir)
            .status();

        if !status_dir.map(|s| s.success()).unwrap_or(false) {
            return (false, Some(format!("Failed to set traversal ACLs on {}", runtime_dir)));
        }
    }

    // grant container root read/write access to the Wayland socket itself (if not already set)
    if !has_acl(&wayland_socket, "100000", "rw-") {
        let status_sock = Command::new("setfacl")
            .arg("-m")
            .arg("u:100000:rw")
            .arg(&wayland_socket)
            .status();

        if !status_sock.map(|s| s.success()).unwrap_or(false) {
            return (false, Some(format!("Failed to set read/write ACLs on {}", wayland_socket)));
        }
    }

    (true, None)
}

#[tauri::command]
pub fn check_drive_configuration() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn check_container_integrity() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn check_container_started() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn check_container_runtime_integrity() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn check_configured_for_personal_use() -> (bool, Option<String>) {
    (true, None)
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

fn has_acl(path: &str, uid: &str, permissions_substring: &str) -> bool {
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