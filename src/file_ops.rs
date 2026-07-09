use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn open_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open File")
        .add_filter("Text Files", &["txt", "md"])
        .add_filter("Scripts", &["ps1", "bat", "cmd", "sh", "py"])
        .add_filter("Executables", &["exe"])
        .add_filter("All Files", &["*"])
        .pick_file()
}

pub fn save_file(path: Option<PathBuf>, suggested_name: Option<String>) -> Option<PathBuf> {
    match path {
        Some(p) => Some(p),
        None => {
            let mut dialog = rfd::FileDialog::new()
                .set_title("Save File")
                .add_filter("Text Files", &["txt", "md"])
                .add_filter("Scripts", &["ps1", "bat", "cmd", "sh", "py"])
                .add_filter("All Files", &["*"]);
            
            if let Some(name) = suggested_name {
                dialog = dialog.set_file_name(&name);
            }
            
            dialog.save_file()
        }
    }
}

pub fn read_file(path: &Path) -> String {
    let content = fs::read_to_string(path).unwrap_or_else(|e| format!("Error reading file: {}", e));
    crate::security::sanitize_content(&content)
}

pub fn write_file(path: &Path, content: &str) -> Result<(), io::Error> {
    fs::write(path, content)
}

pub fn scan_folder(path: &Path) -> Vec<PathBuf> {
    let allowed_extensions = ["txt", "md", "ps1", "bat", "cmd", "exe", "sh", "py"];

    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| allowed_extensions.contains(&ext.to_lowercase().as_str()))
                            .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn open_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open Folder")
        .pick_folder()
}

pub fn pick_save_folder(parent: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Choose Save Folder");
    
    if let Some(parent_path) = parent {
        if parent_path.exists() {
            dialog = dialog.set_directory(parent_path);
        }
    }
    
    let picked = dialog.pick_folder()?;
    
    if let Some(parent_path) = parent {
        let picked_canonical = picked.canonicalize().ok()?;
        let parent_canonical = parent_path.canonicalize().ok()?;
        
        if !picked_canonical.starts_with(&parent_canonical) {
            return None;
        }
        
        Some(picked_canonical)
    } else {
        Some(picked)
    }
}
