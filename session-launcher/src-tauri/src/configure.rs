use std::fs;
use std::path::Path;
use std::process::Command;

use crate::templates::PPE_DEFAULT_CONF;
use crate::checks::has_acl;
use crate::utils::{get_wayland_socket_info, TerminalManager, run_lxc_shell_cmd, get_blacklisted_users};
use crate::session::{SessionState, initial_session_config};

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
pub async fn configure_user(
    username: String, 
    password: String, 
    state: tauri::State<'_, SessionState>
) -> Result<(bool, Option<String>), String> {
    let check_cmd = "getent passwd | awk -F: '$3 >= 1000 && $3 < 65534 {print $1}'";
    let blacklist = get_blacklisted_users();

    if let Ok(existing_users) = run_lxc_shell_cmd(check_cmd) {
        for user in existing_users.lines() {
            let u = user.trim();
            if u.is_empty() { continue; }

            if blacklist.contains(&u.to_string()) {
                // Terminate processes and delete the default user
                let _ = run_lxc_shell_cmd(&format!("pkill -9 -u {}", u));
                let _ = run_lxc_shell_cmd(&format!("userdel -f -r {}", u));
            } else {
                // if somehow the custom user was not yet detected, cache it now
                let mut lock = state.current_user.lock().unwrap();
                *lock = Some(u.to_string());
                return Ok((true, None));
            }
        }
    }

    // create the custom user
    let create_cmd = format!("useradd -m -s /bin/bash {}", username);
    if let Err(e) = run_lxc_shell_cmd(&create_cmd) {
        return Ok((false, Some(format!("Failed to create user: {}", e))));
    }

    // add to the admin group
    let group_cmd = format!("usermod -aG sudo {0} 2>/dev/null || usermod -aG wheel {0} 2>/dev/null", username);
    if let Err(e) = run_lxc_shell_cmd(&group_cmd) {
        return Ok((false, Some(format!("Failed to assign admin permissions: {}", e))));
    }

    // changing the password
    // !! obs !! the password doesn't allow single quotes as it brakes the command
    let pass_cmd = format!("echo '{}:{}' | chpasswd", username, password);
    if let Err(e) = run_lxc_shell_cmd(&pass_cmd) {
        return Ok((false, Some(format!("Failed to set password: {}", e))));
    }

    // make the home directory accessible by the host file manager
    let chmod_cmd = format!("chmod -R o+rwx /home/{}", username);
    if let Err(e) = run_lxc_shell_cmd(&chmod_cmd) {
        return Ok((false, Some(format!("Failed to set portable permissions on home directory: {}", e))));
    }

    // install the prerequisites (currently only for D-Bus configuration)
    if let Err(e) = install_container_prerequisites() {
        return Ok((false, Some(format!("Prerequisite installation failure: {}", e))));
    }

    // initialize the D-Bus session
    let (dbus_ok, dbus_err) = initial_session_config(&username);
    if !dbus_ok {
        return Ok((false, dbus_err));
    }

    let mut lock = state.current_user.lock().unwrap();
    *lock = Some(username);

    Ok((true, None))
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
    for uid in ["100000", "101000"] {
        if !has_acl(&runtime_dir, uid, "--x") {
            let status = Command::new("setfacl")
                .arg("-m").arg(format!("u:{}:x", uid))
                .arg(&runtime_dir)
                .status();
            
            if !status.map(|s| s.success()).unwrap_or(false) {
                return (false, Some(format!("Failed to set traversal ACLs for {} on {}", uid, runtime_dir)));
            }
        }
    }

    // grant container root read/write access to the Wayland socket itself (if not already set)
    for uid in ["100000", "101000"] {
        if !has_acl(&wayland_socket, uid, "rw-") {
            let status = Command::new("setfacl")
                .arg("-m").arg(format!("u:{}:rw", uid))
                .arg(&wayland_socket)
                .status();

            if !status.map(|s| s.success()).unwrap_or(false) {
                return (false, Some(format!("Failed to set R/W ACLs for {} on {}", uid, wayland_socket)));
            }
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

fn install_container_prerequisites() -> Result<(), String> {
    let os_release = run_lxc_shell_cmd("cat /etc/os-release")?;
    
    let mut distro_target = String::new();
    for line in os_release.lines() {
        if line.starts_with("ID_LIKE=") || (line.starts_with("ID=") && distro_target.is_empty()){
            distro_target = line.split('=').nth(1).unwrap_or("").replace('"', "").to_lowercase();
        }
    }

    let install_cmd = if distro_target.contains("debian") || distro_target.contains("ubuntu") {
        "apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y dbus dbus-x11 dconf-gsettings-backend"
    } else if distro_target.contains("fedora") || distro_target.contains("rhel") {
        "dnf install -y dbus dbus-x11 dbus-tools dconf"
    } else if distro_target.contains("arch") {
        "pacman -Sy --noconfirm dbus dconf"
    } else if distro_target.contains("suse") {
        "zypper --non-interactive install dbus-1 dbus-1-x11 dconf"
    } else {
        let _ = std::process::Command::new("notify-send")
            .arg("PPE Launcher Warning")
            .arg("The container distribution family is not supported for automatic D-Bus configuration.")
            .status();
        return Ok(());
    };

    println!("Installing D-Bus prerequisites inside container for the found distribution: {}", distro_target);
    run_lxc_shell_cmd(install_cmd)?;

    // generate the system ID and create a symlink to it in a second standard location
    let _ = run_lxc_shell_cmd("dbus-uuidgen --ensure && ln -sf /var/lib/dbus/machine-id /etc/machine-id");

    Ok(())
}
