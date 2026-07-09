#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod executor;
mod file_ops;
mod security;

use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct FileResult {
    title: String,
    content: String,
    path: Option<String>,
    format: String,
    is_executable: bool,
    runtime_available: bool,
}

#[derive(Serialize)]
struct SaveResult {
    path: String,
    title: String,
    format: String,
    is_executable: bool,
    runtime_available: bool,
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_executable: bool,
}

#[derive(Serialize)]
struct FolderResult {
    folder: String,
    files: Vec<FileInfo>,
}

#[tauri::command]
fn open_file() -> Result<Option<FileResult>, String> {
    let path = match file_ops::open_file() {
        Some(p) => p,
        None => return Ok(None),
    };

    security::validate_path(&path)?;
    security::validate_extension(&path)?;
    security::validate_file_size(&path)?;

    let title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Sem título")
        .to_string();

    let content = file_ops::read_file(&path);

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_executable = security::is_execution_allowed(&path);
    let runtime_available = if format == "py" {
        std::process::Command::new("python")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        true
    };

    Ok(Some(FileResult {
        title,
        content,
        path: Some(path.to_string_lossy().to_string()),
        format: if format == "md" { "markdown".to_string() } else if format == "rtf" { "richtext".to_string() } else if is_executable { "script".to_string() } else { "plaintext".to_string() },
        is_executable,
        runtime_available,
    }))
}

#[tauri::command]
fn open_file_by_path(path: String) -> Result<Option<FileResult>, String> {
    let path = std::path::PathBuf::from(path);

    security::validate_path(&path)?;
    security::validate_extension(&path)?;
    security::validate_file_size(&path)?;

    let title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Sem título")
        .to_string();

    let content = file_ops::read_file(&path);

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_executable = security::is_execution_allowed(&path);
    let runtime_available = if format == "py" {
        std::process::Command::new("python")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        true
    };

    Ok(Some(FileResult {
        title,
        content,
        path: Some(path.to_string_lossy().to_string()),
        format: if format == "md" { "markdown".to_string() } else if format == "rtf" { "richtext".to_string() } else if is_executable { "script".to_string() } else { "plaintext".to_string() },
        is_executable,
        runtime_available,
    }))
}

#[tauri::command]
fn save_file(path: Option<String>, suggested_name: Option<String>, content: String) -> Result<SaveResult, String> {
    let save_path = if let Some(p) = path {
        std::path::PathBuf::from(p)
    } else {
        match file_ops::save_file(None, suggested_name) {
            Some(p) => p,
            None => return Err("Cancelado".to_string()),
        }
    };

    file_ops::write_file(&save_path, &content).map_err(|e| e.to_string())?;

    let title = save_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Sem título")
        .to_string();

    let format = save_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_executable = security::is_execution_allowed(&save_path);
    let runtime_available = if format == "py" {
        std::process::Command::new("python")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        true
    };

    Ok(SaveResult {
        path: save_path.to_string_lossy().to_string(),
        title,
        format: if format == "md" { "markdown".to_string() } else if format == "rtf" { "richtext".to_string() } else if is_executable { "script".to_string() } else { "plaintext".to_string() },
        is_executable,
        runtime_available,
    })
}

#[tauri::command]
fn open_folder() -> Result<Option<FolderResult>, String> {
    let folder = match file_ops::open_folder() {
        Some(p) => p,
        None => return Ok(None),
    };

    let validated = security::validate_path(&folder)?;
    let files = file_ops::scan_folder(&validated);

    let file_infos: Vec<FileInfo> = files
        .into_iter()
        .map(|p| {
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let is_executable = security::is_execution_allowed(&p);
            FileInfo {
                name,
                path: p.to_string_lossy().to_string(),
                is_executable,
            }
        })
        .collect();

    Ok(Some(FolderResult {
        folder: validated.to_string_lossy().to_string(),
        files: file_infos,
    }))
}

#[tauri::command]
fn execute_file(path: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(path);
    security::validate_path(&path)?;
    executor::execute_file(&path)
}

#[tauri::command]
fn stop_execution() -> Result<String, String> {
    executor::stop_execution()
}

#[derive(Serialize)]
struct FolderPickResult {
    path: String,
}

#[tauri::command]
fn pick_save_folder(parent: Option<String>) -> Result<Option<FolderPickResult>, String> {
    let parent_path = parent.map(std::path::PathBuf::from);
    
    let folder = match file_ops::pick_save_folder(parent_path.as_deref()) {
        Some(p) => p,
        None => return Ok(None),
    };
    
    let validated = security::validate_path(&folder)?;
    
    Ok(Some(FolderPickResult {
        path: validated.to_string_lossy().to_string(),
    }))
}

#[derive(Serialize)]
struct FilenameValidationResult {
    valid: bool,
    error: Option<String>,
}

#[tauri::command]
fn validate_filename(filename: String) -> FilenameValidationResult {
    match security::validate_filename(&filename) {
        Ok(()) => FilenameValidationResult {
            valid: true,
            error: None,
        },
        Err(e) => FilenameValidationResult {
            valid: false,
            error: Some(e),
        },
    }
}

#[derive(Serialize)]
struct RuntimeStatus {
    python: bool,
}

#[tauri::command]
fn check_runtimes() -> RuntimeStatus {
    let python = std::process::Command::new("python")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    
    RuntimeStatus { python }
}

#[tauri::command]
fn confirm_close(has_changes: bool) -> bool {
    if !has_changes {
        return true;
    }
    
    rfd::MessageDialog::new()
        .set_title("Alterações não salvas")
        .set_description("Existem alterações não salvas. Deseja fechar mesmo assim?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .set_level(rfd::MessageLevel::Warning)
        .show() == rfd::MessageDialogResult::Yes
}

#[tauri::command]
fn force_close() {
    std::process::exit(0);
}

#[tauri::command]
fn request_close(_app: tauri::AppHandle, has_changes: bool) {
    if !has_changes {
        std::process::exit(0);
    }
    
    let should_close = rfd::MessageDialog::new()
        .set_title("Alterações não salvas")
        .set_description("Existem alterações não salvas. Deseja fechar mesmo assim?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .set_level(rfd::MessageLevel::Warning)
        .show() == rfd::MessageDialogResult::Yes;
    
    if should_close {
        std::process::exit(0);
    }
}

#[tauri::command]
fn set_dirty_state(state: tauri::State<'_, std::sync::Mutex<bool>>, dirty: bool) {
    if let Ok(mut d) = state.lock() {
        *d = dirty;
    }
}

fn main() {
    tauri::Builder::default()
        .manage(std::sync::Mutex::new(false))
        .invoke_handler(tauri::generate_handler![
            open_file,
            open_file_by_path,
            save_file,
            open_folder,
            execute_file,
            stop_execution,
            check_runtimes,
            pick_save_folder,
            validate_filename,
            confirm_close,
            force_close,
            request_close,
            set_dirty_state,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let handle = app.handle().clone();
            
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let state = handle.state::<std::sync::Mutex<bool>>();
                    let has_changes = state.lock().map(|d| *d).unwrap_or(false);
                    
                    if !has_changes {
                        std::process::exit(0);
                    }
                    
                    let should_close = rfd::MessageDialog::new()
                        .set_title("Alterações não salvas")
                        .set_description("Existem alterações não salvas. Deseja fechar mesmo assim?")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .set_level(rfd::MessageLevel::Warning)
                        .show() == rfd::MessageDialogResult::Yes;
                    
                    if should_close {
                        std::process::exit(0);
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
