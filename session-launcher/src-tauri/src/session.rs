use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;
use std::path::Path;
use std::env;
use serde::Serialize;

use crate::templates::UNMOUNT_SCRIPT;
use crate::utils::{get_wayland_socket_info, TerminalManager, FileManager, resolve_icon_path, run_lxc_shell_cmd};


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

#[tauri::command]
pub fn attach_terminal(
    state: tauri::State<'_, SessionState>,
    terminal_manager: tauri::State<'_, TerminalManager>,
) -> (bool, Option<String>) {
    
    let user_lock = state.current_user.lock().unwrap();
    let username = match &*user_lock {
        Some(u) => u.clone(),
        None => return (false, Some("No user is currently logged in or configured.".to_string())),
    };

    let lxc_cmd = format!(
        "exec lxc-attach -n personal-container -P /mnt/ppe/lxc-data --clear-env -- sudo -u {0} bash -c '[ -f /home/{0}/.session_env ] && source /home/{0}/.session_env; cd /home/{0} && exec bash -i'",
        username
    );

    match terminal_manager.launch_detached(&lxc_cmd) {
        Ok(true) => (true, None),
        Ok(false) => (false, Some("Failed to launch detached terminal.".to_string())),
        Err(e) => (false, Some(e)),
    }
}
#[tauri::command]
pub fn open_explorer(
    state: tauri::State<'_, SessionState>,
    file_manager: tauri::State<'_, FileManager>,
) -> (bool, Option<String>) {
    
    let user_lock = state.current_user.lock().unwrap();
    let username = match &*user_lock {
        Some(u) => u.clone(),
        None => return (false, Some("No user is currently logged in.".to_string())),
    };

    let home_path = format!("/mnt/ppe/lxc-data/personal-container/rootfs/home/{}", username);

    match file_manager.open_directory(&home_path) {
        Ok(true) => (true, None),
        Ok(false) => (false, Some("Failed to launch detached file manager.".to_string())),
        Err(e) => (false, Some(e)),
    }
}

#[derive(Serialize, Clone)]
pub struct DesktopApp {
    pub name: String,
    pub exec: String,
    pub icon: String,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
}

#[tauri::command]
pub fn get_installed_apps(state: tauri::State<'_, SessionState>) -> Result<Vec<DesktopApp>, String> {
    let mut apps = Vec::new();
    
    // also check user local applications
    let user_lock = state.current_user.lock().unwrap();
    let username = user_lock.clone().unwrap_or_else(|| "root".to_string());

    let search_dirs = vec![
        "/mnt/ppe/lxc-data/personal-container/rootfs/usr/share/applications".to_string(),
        format!("/mnt/ppe/lxc-data/personal-container/rootfs/home/{}/.local/share/applications", username),
    ];

    for dir in search_dirs {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(app) = parse_desktop_file(&path) {
                        apps.push(app);
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(apps)
}

fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let content = fs::read_to_string(path).ok()?;
    
    let mut name = String::new();
    let mut exec = String::new();
    let mut icon = String::new();
    let mut no_display = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        } else if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }

        if !in_desktop_entry { continue; }

        if line.starts_with("Name=") && name.is_empty() {
            name = line["Name=".len()..].to_string();
        } else if line.starts_with("Exec=") && exec.is_empty() {
            let raw_exec = line["Exec=".len()..].to_string();
            // strip out file/url arguments that break the command -> TODO - check if this brakes apps
            let stripped = raw_exec
                .replace("%u", "").replace("%U", "")
                .replace("%f", "").replace("%F", "")
                .replace("%c", "").replace("%k", "")
                .trim()
                .to_string();
            exec = apply_wayland_flags(&stripped);
        } else if line.starts_with("Icon=") && icon.is_empty() {
            icon = line["Icon=".len()..].to_string();
        } else if line.starts_with("NoDisplay=true") {
            no_display = true;
        }
    }

    // ignore if it's meant to be hidden, or if it lacks a name or exec command
    if no_display || name.is_empty() || exec.is_empty() {
        return None;
    }

    Some(DesktopApp {
        name,
        exec,
        icon: resolve_icon_path(&icon),
        is_custom: false,
    })
}

fn apply_wayland_flags(exec: &str) -> String {
    let binary = exec.split_whitespace().next().unwrap_or("");
    let binary_name = std::path::Path::new(binary)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let needs_ozone_flags = matches!(
        binary_name,
        "google-chrome" | "google-chrome-stable" | "chrome"
        | "chromium" | "chromium-browser"
        | "code" | "code-insiders" | "codium" | "vscodium"
        | "brave-browser" | "brave"
        | "microsoft-edge" | "microsoft-edge-stable"
    );

    if !needs_ozone_flags {
        return exec.to_string();
    }

    let mut parts = exec.splitn(2, char::is_whitespace);
    let bin = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let flags = "--enable-features=UseOzonePlatform --ozone-platform=wayland";
    if rest.is_empty() {
        format!("{} {}", bin, flags)
    } else {
        format!("{} {} {}", bin, flags, rest)
    }
}

#[tauri::command]
pub fn launch_app(
    exec: String,
    with_terminal: bool,
    state: tauri::State<'_, SessionState>,
    terminal_manager: tauri::State<'_, TerminalManager>,
) -> (bool, Option<String>) {
    
    let user_lock = state.current_user.lock().unwrap();
    let username = match &*user_lock {
        Some(u) => u.clone(),
        None => return (false, Some("No user is currently logged in.".to_string())),
    };

    if with_terminal {
        // escape single quotes for safe injection into bash -c '...'
        let escaped_exec = exec.replace('\'', "'\\''");
        
        // so the variables are exported safely && terminal doesn't close instantly if the app crashes
        let container_cmd = format!(
            "[ -f /home/{0}/.session_env ] && source /home/{0}/.session_env; {1} ; echo ''; read -p 'Process exited. Press Enter to close...' -r", 
            username, escaped_exec
        );

        let lxc_cmd = format!(
            "exec lxc-attach -n personal-container -P /mnt/ppe/lxc-data --clear-env -- sudo -u {} bash -c '{}'",
            username, container_cmd
        );

        match terminal_manager.launch_detached(&lxc_cmd) {
            Ok(true) => (true, None),
            Ok(false) => (false, Some("Failed to launch detached terminal.".to_string())),
            Err(e) => (false, Some(e)),
        }
    } else {
        let container_cmd = format!(
            "[ -f /home/{0}/.session_env ] && source /home/{0}/.session_env; {1}", 
            username, exec
        );

        let status = Command::new("lxc-attach")
            .arg("-n").arg("personal-container")
            .arg("-P").arg("/mnt/ppe/lxc-data")
            .arg("--clear-env")
            .arg("--")
            .arg("sudo").arg("-u").arg(&username)
            .arg("bash").arg("-c").arg(&container_cmd)
            .spawn(); 

        match status {
            Ok(_) => (true, None),
            Err(e) => (false, Some(format!("Failed to spawn background app: {}", e))),
        }
    }
}


pub fn initial_session_config(username: &str) -> (bool, Option<String>) {
    let (env_ok, env_err) = initialize_environment_variables(username);
    if !env_ok {
        return (false, env_err);
    }

    let (dbus_ok, dbus_err) = initialize_dbus_session(username);
    if !dbus_ok {
        return (false, dbus_err);
    }

    (true, None)
}

fn initialize_environment_variables(username: &str) -> (bool, Option<String>) {
    let term_env = env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
    
    let env_file = format!("/home/{}/.session_env", username);
    let xdg_dir = format!("/home/{}/.xdg_run", username);

    // create a D-Bus compliant XDG_RUNTIME_DIR and symlink the configured Wayland socket into it
    let _ = run_lxc_shell_cmd(&format!("mkdir -p {}", xdg_dir));
    let _ = run_lxc_shell_cmd(&format!("chown {1}:{1} {0} && chmod 0700 {0}", xdg_dir, username));
    let _ = run_lxc_shell_cmd(&format!("ln -sf /tmp/wayland-0 {}/wayland-0", xdg_dir));

    // always overwrite (with >) the general environment file
    let write_env = format!(
        "cat > {0} << 'EOF'\n\
        export WAYLAND_DISPLAY=wayland-0\n\
        export XDG_RUNTIME_DIR={1}\n\
        export TERM={2}\n\
        export NO_AT_BRIDGE=1\n\
        export ELECTRON_OZONE_PLATFORM_HINT=wayland\n\
        EOF\n\
        chown {3}:{3} {0}",
        env_file, xdg_dir, term_env, username
    );

    if let Err(e) = run_lxc_shell_cmd(&write_env) {
        return (false, Some(format!("Failed to write general environment file: {}", e)));
    }

    (true, None)
}

fn initialize_dbus_session(username: &str) -> (bool, Option<String>) {
    let env_file = format!("/home/{}/.session_env", username);
    let xdg_dir = format!("/home/{}/.xdg_run", username);
    let bus_socket = format!("{}/bus", xdg_dir);

    // kill leftover daemons owned by the user and remove stale socket
    let _ = run_lxc_shell_cmd(&format!("pkill -9 -u {} -x dbus-daemon", username));
    let _ = run_lxc_shell_cmd(&format!("rm -f {}", bus_socket));

    // launch the dbus-daemon at a fixed address
    let launch_cmd = format!(
        "sudo -u {0} bash -c 'XDG_RUNTIME_DIR={1} dbus-daemon --session --address=unix:path={2} --fork </dev/null >/dev/null 2>&1'",
        username, xdg_dir, bus_socket
    );

    if let Err(e) = run_lxc_shell_cmd(&launch_cmd) {
        return (false, Some(format!("Failed to spawn dbus-daemon: {}", e)));
    }

    // append (with >>) the D-Bus address
    let append_env = format!(
        "cat >> {0} << 'EOF'\n\
        export DBUS_SESSION_BUS_ADDRESS='unix:path={1}'\n\
        EOF",
        env_file, bus_socket
    );
    if let Err(e) = run_lxc_shell_cmd(&append_env) {
        return (false, Some(format!("Failed to append dbus env to file: {}", e)));
    }

    // check that the socket exists and is reachable
    let verify_cmd = format!(
        "sudo -u {0} bash -c 'XDG_RUNTIME_DIR={1} DBUS_SESSION_BUS_ADDRESS=unix:path={2} dbus-send --session --dest=org.freedesktop.DBus --type=method_call --print-reply /org/freedesktop/DBus org.freedesktop.DBus.ListNames > /dev/null'",
        username, xdg_dir, bus_socket
    );
    if let Err(e) = run_lxc_shell_cmd(&verify_cmd) {
        return (false, Some(format!("D-Bus session started but is not reachable: {}", e)));
    }

    (true, None)
}