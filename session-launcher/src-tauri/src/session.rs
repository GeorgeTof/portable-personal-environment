use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;

use crate::templates::UNMOUNT_SCRIPT;
use crate::utils::get_wayland_socket_info;


pub struct SessionState {
    pub current_user: Mutex<Option<String>>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            current_user: Mutex::new(None),
        }
    }
}


#[tauri::command]
pub fn start_container() -> (bool, Option<String>) {
    let (runtime_dir, wayland_display) = match get_wayland_socket_info() {
        Ok(info) => info,
        Err(e) => return (false, Some(e)),
    };

    let host_socket_path = format!("{}/{}", runtime_dir, wayland_display);
    
    // bind the dynamic host socket to a static 'tmp/wayland-0' inside the container.
    let mount_entry = format!("{} tmp/wayland-0 none bind,create=file 0 0", host_socket_path);

    let status = Command::new("lxc-start")
        .arg("-n").arg("personal-container")
        .arg("-P").arg("/mnt/ppe/lxc-data")
        .arg("-s").arg(format!("lxc.mount.entry={}", mount_entry))
        .status();

    match status {
        Ok(s) if s.success() => (true, None),
        Ok(_) => (false, Some("lxc-start command failed to execute. Check if the container is already running.".to_string())),
        Err(e) => (false, Some(format!("Failed to spawn lxc-start process: {}", e))),
    }
}

#[tauri::command]
pub fn check_container_started() -> (bool, Option<String>) {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(15);

    while start_time.elapsed() < timeout {
        let status = Command::new("lxc-attach")
            .arg("-n").arg("personal-container")
            .arg("-P").arg("/mnt/ppe/lxc-data")
            .arg("--")
            .arg("/bin/true")
            .status();

        if let Ok(s) = status {
            if s.success() {
                return (true, None);
            }
        }

        thread::sleep(Duration::from_millis(500));
    }

    (false, Some("Container startup timeout: Could not verify initialization after 15 seconds.".to_string()))
}

#[tauri::command]
pub fn stop_container(force: bool) -> (bool, Option<String>) {
    let mut cmd = Command::new("lxc-stop");
    cmd.arg("-n").arg("personal-container")
    .arg("-P").arg("/mnt/ppe/lxc-data");

    if force {
        // flag bypasses the container's init system and kills all processes instantly
        cmd.arg("-k"); 
    } else {
        // flag requests a graceful shutdown and waits up to 10 seconds
        cmd.arg("-t").arg("10"); 
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                return (true, None);
            }

            let combined_output = format!(
                "{} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ).to_lowercase();

            if force && combined_output.contains("is not running") {
                return (false, Some("NOT_RUNNING".to_string()));
            }

            if !force {
                (false, Some("TIMEOUT".to_string()))
            } else {
                (false, Some(combined_output.trim().to_string()))
            }
        },
        Err(e) => (false, Some(format!("Failed to execute lxc-stop: {}", e))),
    }
}

#[tauri::command]
pub fn unmount_drive(app: tauri::AppHandle) -> (bool, Option<String>) {
    let script_path = "/tmp/ppe-unmount-workaround.sh";

    if let Err(e) = fs::write(script_path, UNMOUNT_SCRIPT) {
        return (false, Some(format!("Failed to write unmount script: {}", e)));
    }

    if let Ok(mut perms) = fs::metadata(script_path).map(|m| m.permissions()) {
        perms.set_mode(0b111_101_101);
        if let Err(e) = fs::set_permissions(script_path, perms) {
            return (false, Some(format!("Failed to set permissions on unmount script: {}", e)));
        }
    } else {
        return (false, Some("Failed to read metadata for unmount script.".to_string()));
    }

    match Command::new(script_path) 
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn() 
    {
        Ok(_) => {
            app.exit(0); 
            (true, None)
        },
        Err(e) => (false, Some(format!("Failed to execute unmount script: {}", e))),
    }
}
