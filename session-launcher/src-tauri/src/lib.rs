mod checks;
mod configure;
mod session;
mod templates;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            checks::check_host_config,
            checks::check_drive_configuration,
            checks::check_container_integrity,
            checks::check_container_started,
            checks::check_container_runtime_integrity,
            checks::check_configured_for_personal_use,
            configure::create_environment,
            configure::configure_user,
            session::start_container,
            session::stop_container,
            session::unmount_drive
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
