use std::path::{Path, PathBuf};

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "rtf", "json", "js", "mjs", "cjs", "ts", "tsx", "jsx", "html",
    "css", "xml", "yml", "yaml", "toml", "ini", "cfg", "conf", "log", "csv", "ps1", "bat",
    "cmd", "sh", "bash", "exe", "py", "rb", "pl", "rs", "c", "h", "cpp", "hpp",
];
const EXECUTABLE_EXTENSIONS: &[&str] = &["bat", "cmd", "ps1", "sh", "bash", "exe", "py", "js", "mjs", "cjs"];

pub fn validate_path(path: &Path) -> Result<PathBuf, String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Caminho invalido: {}", e))?;

    for component in canonical.components() {
        if let std::path::Component::Normal(c) = component {
            if let Some(s) = c.to_str() {
                if s.contains("..") || s.contains('~') {
                    return Err("Path traversal detectado".to_string());
                }
            }
        }
    }

    Ok(canonical)
}

pub fn validate_save_path(path: &Path) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .ok_or("Arquivo sem nome")?
        .to_os_string();
    let parent = path
        .parent()
        .ok_or("Arquivo sem diretorio")?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| format!("Caminho invalido: {}", e))?;
    Ok(canonical_parent.join(file_name))
}

pub fn validate_extension(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("Arquivo sem extensao")?;

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!("Extensao nao permitida: .{}", ext));
    }

    Ok(())
}

pub fn validate_file_size(path: &Path) -> Result<(), String> {
    let meta =
        std::fs::metadata(path).map_err(|e| format!("Erro ao ler metadata: {}", e))?;

    if meta.len() > MAX_FILE_SIZE {
        return Err(format!(
            "Arquivo excede o limite de {}MB",
            MAX_FILE_SIZE / (1024 * 1024)
        ));
    }

    Ok(())
}

pub fn is_execution_allowed(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .map(|ext| EXECUTABLE_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

pub fn validate_filename(filename: &str) -> Result<(), String> {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0'];

    if filename.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }

    if filename.chars().count() > 255 {
        return Err("Filename too long (max 255 characters)".to_string());
    }

    if let Some(c) = filename.chars().find(|c| invalid_chars.contains(c)) {
        return Err(format!("Invalid character '{}' in filename", c));
    }

    if filename.chars().any(|c| c.is_control()) {
        return Err("Filename contains control characters".to_string());
    }

    if filename.ends_with('.') || filename.ends_with(' ') {
        return Err("Filename cannot end with '.' or space".to_string());
    }

    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let name_upper = filename
        .split('.')
        .next()
        .unwrap_or("")
        .trim()
        .to_uppercase();
    if reserved_names.contains(&name_upper.as_str()) {
        return Err(format!("Reserved filename '{}' not allowed", filename));
    }

    Ok(())
}

pub fn sanitize_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    for c in content.chars() {
        if c == '\0' {
            continue;
        }
        if !c.is_control() || c == '\n' || c == '\r' || c == '\t' {
            result.push(c);
        }
    }
    result.shrink_to_fit();
    result
}