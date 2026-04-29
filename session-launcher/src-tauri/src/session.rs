#[tauri::command]
pub fn start_container() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn stop_container() -> (bool, Option<String>) {
    (true, None)
}

#[tauri::command]
pub fn unmount_drive() -> (bool, Option<String>) {
    (true, None)
}