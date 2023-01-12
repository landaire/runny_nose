#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use tauri::api::dialog::blocking::FileDialogBuilder;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn open_replays_directory() -> Option<PathBuf> {
    let folder = FileDialogBuilder::new()
        .set_title("Select World of Warships Directory")
        .pick_folder();

    folder
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, open_replays_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
