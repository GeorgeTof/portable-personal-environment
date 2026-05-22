use std::sync::Mutex;
use std::env;
use std::process::{Command};
use std::path::Path;

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
                // obs! not tested on the new gnome terminal! -> todo check!
                Command::new(terminal)
                    .arg("--wait")
                    .arg("--")
                    .arg("bash")
                    .arg("-c")
                    .arg(command)
                    .status()
            },
            "alacritty" | "kitty" | "konsole" => {
                Command::new(terminal)
                    .arg("-e")
                    .arg("bash")
                    .arg("-c")
                    .arg(command)
                    .status()
            },
            _ => {
                // standard fallback (for x-terminal-emulator, xfce4-terminal)
                // obs! xfce4-terminal may have some special parsing rules -> todo check!
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
                    .arg("--wait")
                    .arg("--")
                    .arg("bash")
                    .arg("-c")
                    .arg(command)
                    .spawn()
            },
            "alacritty" | "kitty" |"konsole" => {
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
                "gnome-terminal",      // Standard GNOME
                "konsole",             // Standard KDE
                "xfce4-terminal",      // Standard XFCE
                "alacritty",           // Popular custom
                "kitty",               // Popular custom
                "x-terminal-emulator", // Debian/Ubuntu default wrapper
            ];
            
            for term in terminals {
                if Self::command_exists(term) {
                    println!("[terminal] selected: {}", term);
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

pub struct FileManager {
    cached_manager: Mutex<Option<String>>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            cached_manager: Mutex::new(None),
        }
    }

    pub fn open_directory(&self, path: &str) -> Result<bool, String> {
        let manager = self.get_or_find_manager()?;
        
        let status = Command::new(manager)
            .arg(path)
            .spawn(); // spawn detached

        Ok(status.is_ok())
    }

    fn get_or_find_manager(&self) -> Result<String, String> {
        {
            let mut lock = self.cached_manager.lock().unwrap();
            
            if let Some(mgr) = &*lock {
                return Ok(mgr.clone());
            }

            let managers = [
                "nautilus",  // GNOME
                "dolphin",   // KDE
                "thunar",    // XFCE
                "nemo",      // Cinnamon
                "pcmanfm",   // LXDE/LXQt
                "xdg-open"   // Standard Linux catch-all fallback
            ];
            
            for mgr in managers {
                if Self::command_exists(mgr) {
                    *lock = Some(mgr.to_string());
                    return Ok(mgr.to_string());
                }
            }
        }

        Err("Could not find a supported file manager on the host system.".to_string())
    }

    fn command_exists(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
}

pub fn resolve_icon_path(icon_name: &str) -> String {
    let icon_name = icon_name.trim();
    if icon_name.is_empty() {
        return "".to_string();
    }

    // absolute paths from inside the container
    if icon_name.starts_with('/') {
        let host_path = format!("/mnt/ppe/lxc-data/personal-container/rootfs{}", icon_name);
        if Path::new(&host_path).exists() {
            return host_path;
        }
        // If it starts with / but wasn't found, strip extension and try the fallback logic ???
        // (Sometimes absolute paths omit the extension)
    }

    // standard directories & pixmaps
    let base_icon_dir = "/mnt/ppe/lxc-data/personal-container/rootfs/usr/share/icons/hicolor";
    let pixmaps_dir = "/mnt/ppe/lxc-data/personal-container/rootfs/usr/share/pixmaps";

    // prioritize SVG, then high-res PNGs, down to low-res
    let sizes = ["scalable", "256x256", "128x128", "64x64", "48x48", "32x32"];
    let extensions = [".svg", ".png", ".xpm"];

    // hicolor theme
    for size in sizes {
        for ext in extensions {
            let path = format!("{}/{}/apps/{}{}", base_icon_dir, size, icon_name, ext);
            if Path::new(&path).exists() {
                return path;
            }
        }
    }

    // fallback pixmaps directory
    for ext in extensions {
        let path = format!("{}/{}{}", pixmaps_dir, icon_name, ext);
        if Path::new(&path).exists() {
            return path;
        }
    }

    "".to_string()
}