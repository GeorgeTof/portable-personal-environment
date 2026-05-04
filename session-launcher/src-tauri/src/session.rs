use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;

use crate::utils::get_wayland_socket_info;


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
pub fn stop_container() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn unmount_drive() -> (bool, Option<String>) {
    (true, None)
}