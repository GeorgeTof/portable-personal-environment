use std::process::Command;
use std::env;

pub fn get_wayland_socket_info() -> Result<(String, String), String> {
    let runtime_dir = match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            // fallback to find the runtime directory
            match Command::new("id").arg("-u").output() {
                Ok(out) if out.status.success() => {
                    let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    format!("/run/user/{}", uid)
                }
                _ => return Err("Failed to determine current user ID (id -u failed).".to_string()),
            }
        }
    };
    
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());
    
    Ok((runtime_dir, wayland_display))
}