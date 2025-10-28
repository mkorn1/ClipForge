// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoFile {
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub message: String,
    pub output_path: Option<String>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Export a video file to the specified destination
/// This is a prototype implementation that simply copies the file
#[tauri::command]
fn export_video(source_path: String, destination_path: String) -> Result<ExportResult, String> {
    use std::fs;
    use std::io::Write;
    
    // Read the source file
    let source_data = fs::read(&source_path)
        .map_err(|e| format!("Failed to read source file: {}", e))?;
    
    // Write to destination
    let mut dest_file = fs::File::create(&destination_path)
        .map_err(|e| format!("Failed to create destination file: {}", e))?;
    
    dest_file.write_all(&source_data)
        .map_err(|e| format!("Failed to write to destination: {}", e))?;
    
    // Ensure data is written to disk
    dest_file.sync_all()
        .map_err(|e| format!("Failed to sync file: {}", e))?;
    
    Ok(ExportResult {
        success: true,
        message: "Export completed successfully".to_string(),
        output_path: Some(destination_path),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, export_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
