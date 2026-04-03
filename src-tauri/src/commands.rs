use tauri::command;

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Meeting Assistant is running.", name)
}
