#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod executor;
mod file_ops;
mod security;

use serde::Serialize;
use std::sync::OnceLock;
use tauri::Manager;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

static PYTHON_AVAILABLE: OnceLock<bool> = OnceLock::new();

fn is_python_available() -> bool {
    *PYTHON_AVAILABLE.get_or_init(|| {
        #[cfg(windows)]
        {
            std::process::Command::new("python")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        #[cfg(not(windows))]
        {
            std::process::Command::new("python")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    })
}

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

fn map_format(format: &str, is_executable: bool) -> String {
    if format == "md" {
        "markdown".to_string()
    } else if format == "json" {
        "json".to_string()
    } else if format == "rtf" {
        "richtext".to_string()
    } else if is_executable {
        "script".to_string()
    } else {
        "plaintext".to_string()
    }
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

    let content = file_ops::read_file(&path)?;

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_executable = security::is_execution_allowed(&path);
    let runtime_available = if format == "py" {
        is_python_available()
    } else {
        true
    };

    Ok(Some(FileResult {
        title,
        content,
        path: Some(path.to_string_lossy().to_string()),
        format: map_format(&format, is_executable),
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

    let content = file_ops::read_file(&path)?;

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_executable = security::is_execution_allowed(&path);
    let runtime_available = if format == "py" {
        is_python_available()
    } else {
        true
    };

    Ok(Some(FileResult {
        title,
        content,
        path: Some(path.to_string_lossy().to_string()),
        format: map_format(&format, is_executable),
        is_executable,
        runtime_available,
    }))
}

#[tauri::command]
fn save_file(
    path: Option<String>,
    suggested_name: Option<String>,
    content: String,
) -> Result<SaveResult, String> {
    let save_path = if let Some(p) = path {
        let p = std::path::PathBuf::from(p);
        if !p.exists() {
            let suggested = suggested_name.filter(|name| security::validate_filename(name).is_ok());
            match file_ops::save_file(None, suggested) {
                Some(fp) => {
                    let validated = security::validate_save_path(&fp)?;
                    security::validate_extension(&validated)?;
                    validated
                }
                None => return Err("Cancelado".to_string()),
            }
        } else {
            let validated = security::validate_path(&p)?;
            security::validate_extension(&validated)?;
            validated
        }
    } else {
        let suggested = suggested_name.filter(|name| security::validate_filename(name).is_ok());
        match file_ops::save_file(None, suggested) {
            Some(p) => {
                let validated = security::validate_save_path(&p)?;
                security::validate_extension(&validated)?;
                validated
            }
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
        is_python_available()
    } else {
        true
    };

    Ok(SaveResult {
        path: save_path.to_string_lossy().to_string(),
        title,
        format: map_format(&format, is_executable),
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
fn close_folder() {}

#[tauri::command]
fn scan_folder(path: String) -> Result<Vec<FileInfo>, String> {
    let path = std::path::PathBuf::from(path);
    let validated = security::validate_path(&path)?;
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

    Ok(file_infos)
}

#[tauri::command]
fn execute_file(path: String) -> Result<executor::SessionInfo, String> {
    let path = std::path::PathBuf::from(&path);
    security::validate_path(&path)?;
    security::validate_extension(&path)?;
    if !security::is_execution_allowed(&path) {
        return Err("Tipo de arquivo não executável".to_string());
    }
    executor::start_execution(&path)
}

#[tauri::command]
fn list_executions() -> Result<Vec<executor::SessionInfo>, String> {
    executor::list_sessions()
}

#[tauri::command]
fn list_execution_summaries() -> Result<Vec<executor::SessionSummary>, String> {
    executor::list_session_summaries()
}

#[tauri::command]
fn get_execution(id: u64) -> Result<executor::SessionInfo, String> {
    executor::get_session(id)
}

#[tauri::command]
fn stop_execution(id: Option<u64>) -> Result<String, String> {
    match id {
        Some(session_id) => executor::stop_session(session_id),
        None => executor::stop_all(),
    }
}

#[tauri::command]
fn clear_execution(id: u64) -> Result<String, String> {
    executor::clear_session(id)
}

#[tauri::command]
fn clear_finished_executions() -> Result<String, String> {
    executor::clear_finished()
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
    RuntimeStatus {
        python: is_python_available(),
    }
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
        .show()
        == rfd::MessageDialogResult::Yes
}

#[tauri::command]
fn force_close() {
    let _ = executor::stop_all();
    std::process::exit(0);
}

#[tauri::command]
fn request_close(_app: tauri::AppHandle, has_changes: bool) {
    if !has_changes {
        let _ = executor::stop_all();
        std::process::exit(0);
    }

    let should_close = rfd::MessageDialog::new()
        .set_title("Alterações não salvas")
        .set_description("Existem alterações não salvas. Deseja fechar mesmo assim?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .set_level(rfd::MessageLevel::Warning)
        .show()
        == rfd::MessageDialogResult::Yes;

    if should_close {
        let _ = executor::stop_all();
        std::process::exit(0);
    }
}

#[tauri::command]
fn set_dirty_state(state: tauri::State<'_, std::sync::Mutex<bool>>, dirty: bool) {
    if let Ok(mut d) = state.lock() {
        *d = dirty;
    }
}

#[tauri::command]
fn check_file_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
fn get_locale() -> String {
    std::env::var("LANG")
        .unwrap_or_else(|_| "pt-BR".to_string())
        .split('.').next()
        .unwrap_or("pt-BR")
        .replace('_', "-")
}

fn main() {
    tauri::Builder::default()
        .manage(std::sync::Mutex::new(false))
        .invoke_handler(tauri::generate_handler![
            open_file,
            open_file_by_path,
            save_file,
            open_folder,
            close_folder,
            scan_folder,
            check_file_exists,
            execute_file,
            list_executions,
            list_execution_summaries,
            get_execution,
            stop_execution,
            clear_execution,
            clear_finished_executions,
            check_runtimes,
            pick_save_folder,
            validate_filename,
            confirm_close,
            force_close,
            request_close,
            set_dirty_state,
            get_locale,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let handle = app.handle().clone();

            std::thread::spawn(|| {
                let _ = is_python_available();
            });

            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let state = handle.state::<std::sync::Mutex<bool>>();
                    let has_changes = state.lock().map(|d| *d).unwrap_or(false);

                    if !has_changes {
                        let _ = executor::stop_all();
                        std::process::exit(0);
                    }

                    let should_close = rfd::MessageDialog::new()
                        .set_title("Alterações não salvas")
                        .set_description(
                            "Existem alterações não salvas. Deseja fechar mesmo assim?",
                        )
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .set_level(rfd::MessageLevel::Warning)
                        .show()
                        == rfd::MessageDialogResult::Yes;

                    if should_close {
                        let _ = executor::stop_all();
                        std::process::exit(0);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
