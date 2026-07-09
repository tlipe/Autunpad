use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

static CURRENT_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);

pub fn execute_file(path: &Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or("File has no extension")?
        .to_lowercase();

    let child = match extension.as_str() {
        "bat" | "cmd" => Command::new("cmd.exe")
            .arg("/c")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn cmd.exe: {}", e))?,

        "ps1" => Command::new("powershell.exe")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn powershell.exe: {}", e))?,

        "exe" => Command::new(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn executable: {}", e))?,

        "sh" => Command::new("bash")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn bash: {}", e))?,

        "py" => Command::new("python")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn python: {}", e))?,

        _ => return Err(format!("Unsupported file type: {}", extension)),
    };

    {
        let mut proc = CURRENT_PROCESS.lock().map_err(|e| e.to_string())?;
        *proc = Some(child);
    }

    let timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();

    loop {
        let mut proc = CURRENT_PROCESS.lock().map_err(|e| e.to_string())?;
        let child = proc.as_mut().ok_or_else(|| "Process stopped".to_string())?;

        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output = String::new();

                if let Some(mut stdout) = child.stdout.take() {
                    use std::io::Read;
                    let mut buf = String::new();
                    stdout.read_to_string(&mut buf).ok();
                    output.push_str(&buf);
                }

                if let Some(mut stderr) = child.stderr.take() {
                    use std::io::Read;
                    let mut buf = String::new();
                    stderr.read_to_string(&mut buf).ok();
                    output.push_str(&buf);
                }

                *proc = None;

                if status.success() {
                    return Ok(output);
                } else {
                    return Err(format!(
                        "Process exited with code: {}\n{}",
                        status.code().unwrap_or(-1),
                        output
                    ));
                }
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    child.kill().ok();
                    *proc = None;
                    return Err("Process timed out after 30 seconds".to_string());
                }
                drop(proc);
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                *proc = None;
                return Err(format!("Error waiting for process: {}", e));
            }
        }
    }
}

pub fn stop_execution() -> Result<String, String> {
    let mut proc = CURRENT_PROCESS.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = proc.take() {
        let pid = child.id();
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        child.kill().ok();
        child.wait().ok();
        Ok("Execution stopped".to_string())
    } else {
        Err("No process running".to_string())
    }
}
