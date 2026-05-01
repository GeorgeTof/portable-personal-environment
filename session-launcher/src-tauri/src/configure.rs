use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::env;

use crate::templates::PPE_DEFAULT_CONF;
use crate::checks::has_acl;

// TODO: move to better position, maybe session.rs
pub static HOST_TERMINAL: Mutex<Option<String>> = Mutex::new(None);

#[tauri::command]
pub async fn create_environment() -> (bool, Option<String>) {
    let lxc_data_dir = "/mnt/ppe/lxc-data";
    let config_path = "/mnt/ppe/ppe-default.conf";

    // prepare the lxc-data directory (delete it if it exists, then create fresh)
    if Path::new(lxc_data_dir).exists() {
        if let Err(e) = fs::remove_dir_all(lxc_data_dir) {
            return (false, Some(format!("Failed to clear existing lxc-data directory: {}", e)));
        }
    }
    
    if let Err(e) = fs::create_dir_all(lxc_data_dir) {
        return (false, Some(format!("Failed to create lxc-data directory: {}", e)));
    }

    if let Err(e) = configure_acls(lxc_data_dir) {
        return (false, Some(format!("Failed to set traversal ACLs on lxc-data: {}", e)));
    }

    if let Err(e) = fs::write(config_path, PPE_DEFAULT_CONF) {
        return (false, Some(format!("Failed to write ppe-default.conf: {}", e)));
    }

    let terminal = match get_or_find_terminal() {
        Some(term) => term,
        None => return (false, Some("Could not find a supported terminal emulator on the host system.".to_string())),
    };

    let lxc_cmd = format!(
        "lxc-create -n personal-container -t download -P {} -f {}", lxc_data_dir, config_path
    );

    let success = launch_in_terminal(&terminal, &lxc_cmd);
    
    if !success {
        return (false, Some("The container creation process was cancelled or failed.".to_string()));
    }

    (true, None)
}

#[tauri::command]
pub fn configure_user(_username: String, _password: String) -> (bool, Option<String>) {
    (true, None)
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

pub fn configure_acls(path: &str) -> Result<(), String> {
    let status = Command::new("setfacl")
        .arg("-m")
        .arg("u:100000:x")
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to execute setfacl on {}: {}", path, e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("setfacl returned non-zero status for {}", path))
    }
}



fn get_or_find_terminal() -> Option<String> {
    let mut global_term = HOST_TERMINAL.lock().unwrap();
    
    if let Some(term) = &*global_term {
        return Some(term.clone());
    }

    let terminals = [
        "x-terminal-emulator", // Debian/Ubuntu default wrapper
        "gnome-terminal",      // Standard GNOME
        "konsole",             // Standard KDE
        "xfce4-terminal",      // Standard XFCE
        "alacritty",           // Popular custom
        "kitty"                // Popular custom
    ];
    
    for term in terminals {
        if command_exists(term) {
            *global_term = Some(term.to_string());
            return Some(term.to_string());
        }
    }

    None
}

fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

// todo: maybe move to session or separate better location
fn launch_in_terminal(terminal: &str, command: &str) -> bool {
    // Different terminals require different flags to execute commands.
    // .status() will block the Tauri backend thread until the terminal window is closed
    // perfectly pauses the UI until the user finishes the lxc-create menus
    let status = match terminal {
        "gnome-terminal" => {
            // gnome-terminal deprecated the -e flag in favor of --
            Command::new(terminal)
                .arg("--")
                .arg("bash")
                .arg("-c")
                .arg(command)
                .status()
        },
        "alacritty" | "kitty" => {
            Command::new(terminal)
                .arg("-e")
                .arg("bash")
                .arg("-c")
                .arg(command)
                .status()
        },
        _ => {
            // standard fallback (for x-terminal-emulator, konsole, xfce4-terminal)
            Command::new(terminal)
                .arg("-e")
                .arg(command)
                .status()
        }
    };

    status.map(|s| s.success()).unwrap_or(false)
}