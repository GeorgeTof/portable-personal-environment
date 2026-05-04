use std::fs;
use std::path::Path;
use std::process::Command;

use crate::templates::PPE_DEFAULT_CONF;
use crate::terminal::TerminalManager;
use crate::checks::has_acl;
use crate::utils::get_wayland_socket_info;

#[tauri::command]
// return must be wrapped in result for async command with borrowed parameter, but is deserialized the same in frontend
pub async fn create_environment(terminal_manager: tauri::State<'_, TerminalManager>) -> Result<(bool, Option<String>), String> {
    let lxc_data_dir = "/mnt/ppe/lxc-data";
    let config_path = "/mnt/ppe/ppe-default.conf";

    // prepare the lxc-data directory (delete it if it exists, then create fresh)
    if Path::new(lxc_data_dir).exists() {
        if let Err(e) = fs::remove_dir_all(lxc_data_dir) {
            return Ok((false, Some(format!("Failed to clear existing lxc-data directory: {}", e))));
        }
    }
    
    if let Err(e) = fs::create_dir_all(lxc_data_dir) {
        return Ok((false, Some(format!("Failed to create lxc-data directory: {}", e))));
    }

    if let Err(e) = configure_acls(lxc_data_dir) {
        return Ok((false, Some(format!("Failed to set traversal ACLs on lxc-data: {}", e))));
    }

    if let Err(e) = fs::write(config_path, PPE_DEFAULT_CONF) {
        return Ok((false, Some(format!("Failed to write ppe-default.conf: {}", e))));
    }

    let lxc_cmd = format!(
        "lxc-create -n personal-container -t download -P {} -f {}", 
        lxc_data_dir, config_path
    );

    let result = match terminal_manager.launch_and_wait(&lxc_cmd) {
        Ok(true) => (true, None),
        Ok(false) => (false, Some("The container creation process was cancelled or failed.".to_string())),
        Err(e) => (false, Some(e)),
    };

    Ok(result)
}

#[tauri::command]
pub fn configure_user(_username: String, _password: String) -> (bool, Option<String>) {
    (true, None)
}



pub fn configure_wayland_access() -> (bool, Option<String>) {
    let (runtime_dir, wayland_display) = match get_wayland_socket_info() {
        Ok(info) => info,
        Err(err_msg) => return (false, Some(err_msg)),
    };

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