// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingResult {
    pub process_id: u32,
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopRecordingResult {
    pub success: bool,
    pub file_path: String,
    pub message: String,
}

// Global storage for active recording processes
// Maps process ID to process handle, output path, and stderr handle
lazy_static::lazy_static! {
    static ref RECORDING_PROCESSES: Mutex<HashMap<u32, (std::process::Child, String)>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub has_permission: bool,
    pub message: String,
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

/// Start screen recording using FFmpeg
/// Returns a process ID that can be used to stop the recording
#[tauri::command]
fn start_screen_recording(output_path: Option<String>) -> Result<RecordingResult, String> {
    // Generate output path if not provided
    let output = if let Some(path) = output_path {
        path
    } else {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_dir = std::env::temp_dir();
        temp_dir
            .join(format!("clipforge-recording-{}.mp4", timestamp))
            .to_str()
            .ok_or("Failed to create temp file path")?
            .to_string()
    };

    // Check if FFmpeg is available
    let ffmpeg_check = Command::new("ffmpeg")
        .arg("-version")
        .output();
    
    match ffmpeg_check {
        Ok(_) => {},
        Err(_) => return Err("FFmpeg is not installed or not found in PATH. Please install FFmpeg to use screen recording.".to_string()),
    }

    // Construct FFmpeg command for macOS using avfoundation
    // Screen capture devices start at index 4 (Capture screen 0), 5 (Capture screen 1), etc.
    // Format: ffmpeg -f avfoundation -i "4:" -r 30 -c:v libx264 -preset fast -crf 23 -pix_fmt yuv420p output.mp4
    // "4:" means screen capture device 4 (first screen), no audio device
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-f")
        .arg("avfoundation")
        .arg("-capture_cursor")
        .arg("1")  // Capture cursor
        .arg("-framerate")
        .arg("30")  // Input framerate
        .arg("-i")
        .arg("4:")  // Screen capture device 4 (Capture screen 0), no audio
        .arg("-r")
        .arg("30")  // Output framerate
        .arg("-c:v")
        .arg("libx264")  // Video codec
        .arg("-preset")
        .arg("fast")  // Encoding speed
        .arg("-crf")
        .arg("23")  // Quality (lower = better, 18-28 is reasonable range)
        .arg("-pix_fmt")
        .arg("yuv420p")  // Pixel format for compatibility
        .arg("-y")  // Overwrite output file
        .arg(&output)
        // Capture stderr to log errors for debugging
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null());

    // Spawn the FFmpeg process
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start FFmpeg process: {}. Make sure FFmpeg is installed and available in PATH.", e))?;

    // Give FFmpeg a moment to initialize and check if it's still running
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // Check if process immediately crashed
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process already exited - try to read stderr for error info
            let error_msg = if let Some(mut stderr) = child.stderr.take() {
                use std::io::Read;
                let mut error_output = String::new();
                let _ = stderr.read_to_string(&mut error_output);
                if !error_output.is_empty() {
                    format!("FFmpeg exited immediately with status {:?}. Error output: {}", status, error_output)
                } else {
                    format!("FFmpeg exited immediately with status {:?}", status)
                }
            } else {
                format!("FFmpeg exited immediately with status {:?}", status)
            };
            return Err(error_msg);
        }
        Ok(None) => {
            // Process is still running, good!
        }
        Err(e) => {
            return Err(format!("Failed to check FFmpeg process status: {}", e));
        }
    }

    // Generate a unique process ID
    let process_id = child.id();

    // Store the process handle and output path
    let mut processes = RECORDING_PROCESSES.lock()
        .map_err(|e| format!("Failed to lock recording processes: {}", e))?;
    
    processes.insert(process_id, (child, output.clone()));

    Ok(RecordingResult {
        process_id,
        output_path: output,
    })
}

/// Start webcam recording using FFmpeg
/// Returns a process ID that can be used to stop the recording
#[tauri::command]
fn start_webcam_recording(output_path: Option<String>, device_index: Option<u32>) -> Result<RecordingResult, String> {
    // Generate output path if not provided
    let output = if let Some(path) = output_path {
        path
    } else {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_dir = std::env::temp_dir();
        temp_dir
            .join(format!("clipforge-webcam-{}.mp4", timestamp))
            .to_str()
            .ok_or("Failed to create temp file path")?
            .to_string()
    };

    // Check if FFmpeg is available
    let ffmpeg_check = Command::new("ffmpeg")
        .arg("-version")
        .output();
    
    match ffmpeg_check {
        Ok(_) => {},
        Err(_) => return Err("FFmpeg is not installed or not found in PATH. Please install FFmpeg to use webcam recording.".to_string()),
    }

    // Use device index 0 by default (first webcam), or user-specified
    let device_idx = device_index.unwrap_or(0);
    let device_string = format!("{}:", device_idx);

    // Construct FFmpeg command for macOS using avfoundation
    // Format: ffmpeg -f avfoundation -i "0:" -r 30 -c:v libx264 -preset fast -crf 23 -pix_fmt yuv420p output.mp4
    // "0:" means video device 0 (first webcam), no audio device
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-f")
        .arg("avfoundation")
        .arg("-framerate")
        .arg("30")  // Input framerate
        .arg("-video_size")
        .arg("1280x720")  // Common webcam resolution, can be made configurable
        .arg("-i")
        .arg(&device_string)  // Webcam device index
        .arg("-r")
        .arg("30")  // Output framerate
        .arg("-c:v")
        .arg("libx264")  // Video codec
        .arg("-preset")
        .arg("fast")  // Encoding speed
        .arg("-crf")
        .arg("23")  // Quality (lower = better, 18-28 is reasonable range)
        .arg("-pix_fmt")
        .arg("yuv420p")  // Pixel format for compatibility
        .arg("-y")  // Overwrite output file
        .arg(&output)
        // Capture stderr to log errors for debugging
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null());

    // Spawn the FFmpeg process
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start FFmpeg process: {}. Make sure FFmpeg is installed and available in PATH.", e))?;

    // Give FFmpeg a moment to initialize and check if it's still running
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // Check if process immediately crashed
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process already exited - try to read stderr for error info
            let error_msg = if let Some(mut stderr) = child.stderr.take() {
                use std::io::Read;
                let mut error_output = String::new();
                let _ = stderr.read_to_string(&mut error_output);
                if !error_output.is_empty() {
                    format!("FFmpeg exited immediately with status {:?}. Error output: {}", status, error_output)
                } else {
                    format!("FFmpeg exited immediately with status {:?}", status)
                }
            } else {
                format!("FFmpeg exited immediately with status {:?}", status)
            };
            return Err(error_msg);
        }
        Ok(None) => {
            // Process is still running, good!
        }
        Err(e) => {
            return Err(format!("Failed to check FFmpeg process status: {}", e));
        }
    }

    // Generate a unique process ID
    let process_id = child.id();

    // Store the process handle and output path
    let mut processes = RECORDING_PROCESSES.lock()
        .map_err(|e| format!("Failed to lock recording processes: {}", e))?;
    
    processes.insert(process_id, (child, output.clone()));

    Ok(RecordingResult {
        process_id,
        output_path: output,
    })
}

/// Stop a screen recording process
/// Returns the path to the saved recording file
#[tauri::command]
fn stop_screen_recording(process_id: u32) -> Result<StopRecordingResult, String> {
    let mut processes = RECORDING_PROCESSES.lock()
        .map_err(|e| format!("Failed to lock recording processes: {}", e))?;

    // Find and remove the process
    let (mut child, output_path) = processes.remove(&process_id)
        .ok_or_else(|| format!("Recording process with ID {} not found", process_id))?;

    // Get the actual child process ID (might be different from stored process_id)
    let child_pid = child.id();
    
    // Try to gracefully stop FFmpeg first
    #[cfg(unix)]
    {
        // On Unix, try to send SIGINT for graceful shutdown using the actual child PID
        let pid = nix::unistd::Pid::from_raw(child_pid as i32);
        if nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGINT).is_ok() {
            // Give FFmpeg a moment to flush buffers
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            // Check if process already exited gracefully
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process already exited, check file
                    if std::path::Path::new(&output_path).exists() {
                        return Ok(StopRecordingResult {
                            success: true,
                            file_path: output_path,
                            message: "Recording saved successfully".to_string(),
                        });
                    }
                }
                Ok(None) => {
                    // Process still running, continue to kill
                }
                Err(e) => {
                    eprintln!("Error checking process status: {}", e);
                }
            }
        }
    }

    // Kill the process if it's still running
    let _ = child.kill();

    // Wait for the process to finish
    let wait_result = child.wait();
    
    // Try to read stderr for error messages
    let mut stderr_output = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        use std::io::Read;
        let _ = stderr.read_to_string(&mut stderr_output);
    }

    // Give more time for file system to sync (FFmpeg might still be flushing)
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Check if the output file exists
    if std::path::Path::new(&output_path).exists() {
        // Verify file is not empty
        if let Ok(metadata) = std::fs::metadata(&output_path) {
            if metadata.len() > 0 {
                return Ok(StopRecordingResult {
                    success: true,
                    file_path: output_path,
                    message: "Recording saved successfully".to_string(),
                });
            } else {
                return Err(format!(
                    "Recording file exists but is empty (0 bytes). FFmpeg may have failed to record. Stderr: {}",
                    if stderr_output.is_empty() { "No error output".to_string() } else { stderr_output }
                ));
            }
        }
    }
    
    // File doesn't exist - provide detailed error information
    let error_details = match wait_result {
        Ok(status) => {
            let status_str = format!("{:?}", status);
            let mut details = format!("Process exited with status: {}", status_str);
            if !stderr_output.is_empty() {
                details.push_str(&format!("\nFFmpeg stderr output:\n{}", stderr_output));
            }
            details
        },
        Err(e) => format!("Failed to wait for process: {}", e),
    };
    
    Err(format!(
        "Recording file not found at '{}'.\n{}",
        output_path, error_details
    ))
}

/// Check screen recording permission status on macOS
/// Note: Direct permission checking requires Objective-C/Swift interop, so this is a placeholder
#[tauri::command]
fn check_screen_recording_permission() -> Result<PermissionStatus, String> {
    #[cfg(target_os = "macos")]
    {
        // On macOS, we can't directly check permissions from Rust without FFI
        // The actual permission check will happen when getDisplayMedia() is called
        // For now, return a placeholder that indicates permission should be checked by the frontend
        Ok(PermissionStatus {
            has_permission: false, // Assume false, frontend will verify via getDisplayMedia()
            message: "Permission status cannot be determined from backend. Frontend will check via getDisplayMedia() API.".to_string(),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(PermissionStatus {
            has_permission: true,
            message: "Screen recording permissions not applicable on this platform".to_string(),
        })
    }
}

/// Start simultaneous screen + webcam recording with picture-in-picture overlay
/// Returns a process ID that can be used to stop the recording
#[tauri::command]
fn start_screen_webcam_recording(
    output_path: Option<String>,
    webcam_device_index: Option<u32>,
    pip_position: Option<String>, // "bottom-right", "bottom-left", "top-right", "top-left"
    pip_size: Option<String>,      // e.g., "320:240" or "25%"
) -> Result<RecordingResult, String> {
    // Generate output path if not provided
    let output = if let Some(path) = output_path {
        path
    } else {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_dir = std::env::temp_dir();
        temp_dir
            .join(format!("clipforge-pip-{}.mp4", timestamp))
            .to_str()
            .ok_or("Failed to create temp file path")?
            .to_string()
    };

    // Check if FFmpeg is available
    let ffmpeg_check = Command::new("ffmpeg")
        .arg("-version")
        .output();
    
    match ffmpeg_check {
        Ok(_) => {},
        Err(_) => return Err("FFmpeg is not installed or not found in PATH. Please install FFmpeg to use screen recording.".to_string()),
    }

    // Use device index 0 by default for webcam, or user-specified
    let webcam_idx = webcam_device_index.unwrap_or(0);
    let screen_device = "4:";  // Screen capture device
    let webcam_device = format!("{}:", webcam_idx);

    // Default PiP settings
    let pip_width = "320";
    let pip_height = "240";
    let position = pip_position.as_deref().unwrap_or("bottom-right");
    
    // Calculate overlay position based on desired corner
    let overlay_pos = match position {
        "bottom-right" => "W-w-10:H-h-10",  // 10px from right and bottom
        "bottom-left" => "10:H-h-10",       // 10px from left and bottom
        "top-right" => "W-w-10:10",         // 10px from right and top
        "top-left" => "10:10",              // 10px from left and top
        _ => "W-w-10:H-h-10",               // Default to bottom-right
    };

    // Construct FFmpeg command with filter_complex for PiP overlay
    // Input 0: Screen capture (device 4)
    // Input 1: Webcam (device 0 or specified)
    // Filter: Scale webcam and overlay on screen
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-f")
        .arg("avfoundation")
        .arg("-capture_cursor")
        .arg("1")  // Capture cursor on screen
        .arg("-framerate")
        .arg("30")
        .arg("-i")
        .arg(screen_device)  // Input 0: Screen
        .arg("-f")
        .arg("avfoundation")
        .arg("-framerate")
        .arg("30")
        .arg("-video_size")
        .arg("1280x720")  // Webcam resolution (will be scaled down)
        .arg("-i")
        .arg(&webcam_device)  // Input 1: Webcam
        .arg("-filter_complex")
        .arg(format!(
            "[1:v]scale={}:{}[webcam];[0:v][webcam]overlay={}[v]",
            pip_width, pip_height, overlay_pos
        ))
        .arg("-map")
        .arg("[v]")  // Map the filtered output
        .arg("-r")
        .arg("30")  // Output framerate
        .arg("-c:v")
        .arg("libx264")  // Video codec
        .arg("-preset")
        .arg("fast")  // Encoding speed
        .arg("-crf")
        .arg("23")  // Quality
        .arg("-pix_fmt")
        .arg("yuv420p")  // Pixel format for compatibility
        .arg("-y")  // Overwrite output file
        .arg(&output)
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null());

    // Spawn the FFmpeg process
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start FFmpeg process: {}. Make sure FFmpeg is installed and available in PATH.", e))?;

    // Give FFmpeg a moment to initialize
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // Check if process immediately crashed
    match child.try_wait() {
        Ok(Some(status)) => {
            let error_msg = if let Some(mut stderr) = child.stderr.take() {
                use std::io::Read;
                let mut error_output = String::new();
                let _ = stderr.read_to_string(&mut error_output);
                if !error_output.is_empty() {
                    format!("FFmpeg exited immediately with status {:?}. Error output: {}", status, error_output)
                } else {
                    format!("FFmpeg exited immediately with status {:?}", status)
                }
            } else {
                format!("FFmpeg exited immediately with status {:?}", status)
            };
            return Err(error_msg);
        }
        Ok(None) => {
            // Process is still running, good!
        }
        Err(e) => {
            return Err(format!("Failed to check FFmpeg process status: {}", e));
        }
    }

    // Generate a unique process ID
    let process_id = child.id();

    // Store the process handle and output path
    let mut processes = RECORDING_PROCESSES.lock()
        .map_err(|e| format!("Failed to lock recording processes: {}", e))?;
    
    processes.insert(process_id, (child, output.clone()));

    Ok(RecordingResult {
        process_id,
        output_path: output,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            export_video,
            start_screen_recording,
            start_webcam_recording,
            start_screen_webcam_recording,  // Add this
            stop_screen_recording,
            check_screen_recording_permission
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
