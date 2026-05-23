mod checks;
mod configure;
mod session;
mod templates;
mod utils;

use utils::TerminalManager;
use session::SessionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(TerminalManager::new())
        .manage(utils::FileManager::new())
        .manage(SessionState::new())
        .manage(session::AppWatcherState::new())
        .invoke_handler(tauri::generate_handler![
            checks::check_host_config,
            checks::check_drive_configuration,
            checks::check_container_integrity,
            checks::check_container_runtime_integrity,
            checks::check_configured_for_personal_use,
            configure::create_environment,
            configure::configure_user,
            session::start_container,
            session::check_container_started,
            session::stop_container,
            session::unmount_drive,
            session::attach_terminal,
            session::open_explorer,
            session::open_shared,
            session::get_installed_apps,
            session::start_app_watcher,
            session::launch_app,
            utils::get_blacklisted_users,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
