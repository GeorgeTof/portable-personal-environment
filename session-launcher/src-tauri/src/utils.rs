use std::sync::Mutex;
use std::env;
use std::process::{Command};

fn get_blacklisted_users_internal() -> Vec<&'static str> {
    vec![
        "ubuntu", "debian", "fedora", "alpine", "arch", 
        "mint", "kali", "root", "admin", "guest"
    ]
}

#[tauri::command]
pub fn get_blacklisted_users() -> Vec<String> {
    get_blacklisted_users_internal().into_iter().map(String::from).collect()
}

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


pub struct TerminalManager {
    cached_terminal: Mutex<Option<String>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            cached_terminal: Mutex::new(None),
        }
    }

    pub fn launch_and_wait(&self, command: &str) -> Result<bool, String> {
        let terminal = self.get_or_find_terminal()?;
        
        // Different terminals require different flags to execute commands.
        // .status() will block the Tauri backend thread until the terminal window is closed
        // perfectly pauses the UI until the user finishes the lxc-create menus
        let status = match terminal.as_str() {
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

        Ok(status.map(|s| s.success()).unwrap_or(false))
    }

    pub fn launch_detached(&self, command: &str) -> Result<bool, String> {
        let terminal = self.get_or_find_terminal()?;
        
        // using .spawn() here instead of .status()
        let status = match terminal.as_str() {
            "gnome-terminal" => {
                Command::new(terminal)
                    .arg("--")
                    .arg("bash")
                    .arg("-c")
                    .arg(command)
                    .spawn()
            },
            "alacritty" | "kitty" => {
                Command::new(terminal)
                    .arg("-e")
                    .arg("bash")
                    .arg("-c")
                    .arg(command)
                    .spawn()
            },
            _ => {
                Command::new(terminal)
                    .arg("-e")
                    .arg(command)
                    .spawn()
            }
        };

        Ok(status.is_ok())
    }


    fn get_or_find_terminal(&self) -> Result<String, String> {
        // open a new scope to ensure the lock drops immediately after cloning the string
        {
            let mut lock = self.cached_terminal.lock().unwrap();
            
            if let Some(term) = &*lock {
                return Ok(term.clone());
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
                if Self::command_exists(term) {
                    *lock = Some(term.to_string());
                    return Ok(term.to_string());
                }
            }
        }

        Err("Could not find a supported terminal emulator on the host system.".to_string())
    }

    fn command_exists(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

}

pub fn run_lxc_shell_cmd(cmd_str: &str) -> Result<String, String> {
    let output = Command::new("lxc-attach")
        .arg("-n").arg("personal-container")
        .arg("-P").arg("/mnt/ppe/lxc-data")
        .arg("--")
        .arg("sh").arg("-c").arg(cmd_str)
        .output()
        .map_err(|e| format!("Failed to execute lxc-attach: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}