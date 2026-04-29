#[tauri::command]
pub fn create_environment() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn configure_user(_username: String, _password: String) -> (bool, Option<String>) {
    (true, None)
}