use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const MAX_CONCURRENT: usize = 8;
const MAX_OUTPUT: usize = 32 * 1024;
const DEFAULT_TIMEOUT_SECS: u64 = 300;
const MAX_OUTPUT_LINES: usize = 400;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static SESSIONS: OnceLock<Mutex<HashMap<u64, Arc<Session>>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<u64, Arc<Session>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Running,
    Done,
    Error,
    Stopped,
    Timeout,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Running => "running",
            Status::Done => "done",
            Status::Error => "error",
            Status::Stopped => "stopped",
            Status::Timeout => "timeout",
        }
    }
}

struct Session {
    id: u64,
    name: Box<str>,
    runtime: Box<str>,
    path: Box<str>,
    status: Mutex<Status>,
    output: Mutex<String>,
    exit_code: Mutex<Option<i32>>,
    pid: AtomicU32,
    started_ms: u64,
}

#[derive(Serialize, Clone)]
pub struct SessionInfo {
    pub id: u64,
    pub name: String,
    pub runtime: String,
    pub path: String,
    pub status: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub started_ms: u64,
}

#[derive(Serialize, Clone)]
pub struct SessionSummary {
    pub id: u64,
    pub name: String,
    pub runtime: String,
    pub path: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub started_ms: u64,
    pub output_len: usize,
}

fn runtime_label(ext: &str) -> &'static str {
    match ext {
        "ps1" => "PowerShell",
        "bat" | "cmd" => "CMD",
        "py" => "Python",
        "sh" => "Bash",
        "exe" => "Native",
        "js" | "mjs" | "cjs" => "Node",
        _ => "Process",
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn append_output(session: &Session, chunk: &str) {
    if chunk.is_empty() {
        return;
    }
    if let Ok(mut out) = session.output.lock() {
        if out.len() >= MAX_OUTPUT {
            return;
        }
        let room = MAX_OUTPUT.saturating_sub(out.len());
        if chunk.len() <= room {
            out.push_str(chunk);
        } else {
            out.push_str(&chunk[..room]);
            out.push_str("\n...[truncated]...");
        }
        
        let lines: Vec<&str> = out.lines().collect();
        if lines.len() > MAX_OUTPUT_LINES {
            let start_line = lines.len() - MAX_OUTPUT_LINES;
            *out = lines[start_line..].join("\n");
        }
    }
}

fn set_status(session: &Session, status: Status, code: Option<i32>) {
    if let Ok(mut s) = session.status.lock() {
        if *s == Status::Stopped {
            return;
        }
        *s = status;
    }
    if let Ok(mut c) = session.exit_code.lock() {
        *c = code;
    }
}

fn kill_pid(pid: u32) {
    if pid == 0 {
        return;
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn spawn_for_extension(path: &Path, extension: &str) -> Result<(Child, &'static str), String> {
    let workdir = path.parent().filter(|p| p.is_dir());

    match extension {
        "bat" | "cmd" => {
            let mut cmd = Command::new("cmd.exe");
            let quoted = format!("\"{}\"", path.display());
            cmd.args(["/s", "/c", &quoted]);
            if let Some(dir) = workdir {
                cmd.current_dir(dir);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn cmd.exe: {}", e))?;
            Ok((child, runtime_label(extension)))
        }
        "ps1" => {
            let mut cmd = Command::new("powershell.exe");
            let script = format!("& '{}'", path.to_string_lossy().replace('\'', "''"));
            cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command"])
                .arg(&script);
            if let Some(dir) = workdir {
                cmd.current_dir(dir);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn powershell.exe: {}", e))?;
            Ok((child, runtime_label(extension)))
        }
        "exe" => {
            let mut cmd = Command::new(path);
            if let Some(dir) = workdir {
                cmd.current_dir(dir);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn executable: {}", e))?;
            Ok((child, runtime_label(extension)))
        }
        "sh" => {
            let mut cmd = Command::new("bash");
            cmd.arg(path);
            if let Some(dir) = workdir {
                cmd.current_dir(dir);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn bash: {}", e))?;
            Ok((child, runtime_label(extension)))
        }
        "py" => {
            let mut cmd = Command::new("python");
            cmd.arg(path);
            if let Some(dir) = workdir {
                cmd.current_dir(dir);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn python: {}", e))?;
            Ok((child, runtime_label(extension)))
        }
        "js" | "mjs" | "cjs" => {
            let mut cmd = Command::new("node");
            cmd.arg(path);
            if let Some(dir) = workdir {
                cmd.current_dir(dir);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            let child = cmd
                .spawn()
                .map_err(|e| format!("Failed to spawn node: {}", e))?;
            Ok((child, runtime_label(extension)))
        }
        _ => Err(format!("Unsupported file type: {}", extension)),
    }
}

fn session_to_info(session: &Session) -> SessionInfo {
    SessionInfo {
        id: session.id,
        name: session.name.to_string(),
        runtime: session.runtime.to_string(),
        path: session.path.to_string(),
        status: session
            .status
            .lock()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|_| "error".to_string()),
        output: session
            .output
            .lock()
            .map(|o| o.clone())
            .unwrap_or_default(),
        exit_code: session.exit_code.lock().ok().and_then(|c| *c),
        started_ms: session.started_ms,
    }
}

fn session_to_summary(session: &Session) -> SessionSummary {
    SessionSummary {
        id: session.id,
        name: session.name.to_string(),
        runtime: session.runtime.to_string(),
        path: session.path.to_string(),
        status: session
            .status
            .lock()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|_| "error".to_string()),
        exit_code: session.exit_code.lock().ok().and_then(|c| *c),
        started_ms: session.started_ms,
        output_len: session.output.lock().map(|o| o.len()).unwrap_or(0),
    }
}


fn running_count(map: &HashMap<u64, Arc<Session>>) -> usize {
    map.values()
        .filter(|s| {
            s.status
                .lock()
                .map(|st| *st == Status::Running)
                .unwrap_or(false)
        })
        .count()
}

pub fn start_execution(path: &Path) -> Result<SessionInfo, String> {
    if !crate::security::is_execution_allowed(path) {
        return Err("Execution not allowed for this file type".to_string());
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or("File has no extension")?
        .to_lowercase();

    {
        let sessions = sessions().lock().map_err(|e| e.to_string())?;
        if running_count(&sessions) >= MAX_CONCURRENT {
            return Err(format!(
                "Too many parallel terminals (max {})",
                MAX_CONCURRENT
            ));
        }
    }

    let (mut child, runtime) = spawn_for_extension(path, &extension)?;
    let pid = child.id();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("script")
        .to_string();

    let session = Arc::new(Session {
        id,
        name: name.into_boxed_str(),
        runtime: runtime.to_string().into_boxed_str(),
        path: path.to_string_lossy().to_string().into_boxed_str(),
        status: Mutex::new(Status::Running),
        output: Mutex::new(String::with_capacity(1024)),
        exit_code: Mutex::new(None),
        pid: AtomicU32::new(pid),
        started_ms: now_ms(),
    });

    {
        let mut sessions = sessions().lock().map_err(|e| e.to_string())?;
        sessions.insert(id, Arc::clone(&session));
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let session_out = Arc::clone(&session);
    let session_err = Arc::clone(&session);
    let session_wait = Arc::clone(&session);

    if let Some(out) = stdout {
        thread::spawn(move || {
            let mut reader = BufReader::new(out);
            let mut buf = String::new();
            loop {
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => append_output(&session_out, &buf),
                    Err(_) => break,
                }
            }
        });
    }

    if let Some(err) = stderr {
        thread::spawn(move || {
            let mut reader = BufReader::new(err);
            let mut buf = String::new();
            loop {
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => append_output(&session_err, &buf),
                    Err(_) => break,
                }
            }
        });
    }

    thread::spawn(move || {
        let start = Instant::now();
        let timeout = Duration::from_secs(DEFAULT_TIMEOUT_SECS);

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Drain any leftover pipes briefly
                    thread::sleep(Duration::from_millis(50));
                    let code = status.code();
                    if status.success() {
                        set_status(&session_wait, Status::Done, code);
                    } else {
                        set_status(&session_wait, Status::Error, code);
                        if let Some(c) = code {
                            append_output(
                                &session_wait,
                                &format!("\n[exit code: {}]\n", c),
                            );
                        }
                    }
                    session_wait.pid.store(0, Ordering::Relaxed);
                    break;
                }
                Ok(None) => {
                    let stopped = session_wait
                        .status
                        .lock()
                        .map(|s| *s == Status::Stopped)
                        .unwrap_or(false);
                    if stopped {
                        kill_pid(session_wait.pid.load(Ordering::Relaxed));
                        let _ = child.kill();
                        let _ = child.wait();
                        session_wait.pid.store(0, Ordering::Relaxed);
                        break;
                    }
                    if start.elapsed() > timeout {
                        kill_pid(session_wait.pid.load(Ordering::Relaxed));
                        let _ = child.kill();
                        let _ = child.wait();
                        set_status(&session_wait, Status::Timeout, None);
                        append_output(
                            &session_wait,
                            &format!(
                                "\n[timeout after {}s]\n",
                                DEFAULT_TIMEOUT_SECS
                            ),
                        );
                        session_wait.pid.store(0, Ordering::Relaxed);
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    set_status(&session_wait, Status::Error, None);
                    append_output(&session_wait, &format!("\n[wait error: {}]\n", e));
                    session_wait.pid.store(0, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    Ok(session_to_info(&session))
}

pub fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let sessions = sessions().lock().map_err(|e| e.to_string())?;
    let mut list: Vec<SessionInfo> = sessions.values().map(|s| session_to_info(s)).collect();
    list.sort_by_key(|s| s.id);
    Ok(list)
}

pub fn list_session_summaries() -> Result<Vec<SessionSummary>, String> {
    let sessions = sessions().lock().map_err(|e| e.to_string())?;
    let mut list: Vec<SessionSummary> = sessions.values().map(|s| session_to_summary(s)).collect();
    list.sort_by_key(|s| s.id);
    Ok(list)
}

pub fn get_session(id: u64) -> Result<SessionInfo, String> {
    let sessions = sessions().lock().map_err(|e| e.to_string())?;
    sessions
        .get(&id)
        .map(|s| session_to_info(s))
        .ok_or_else(|| "Session not found".to_string())
}

pub fn stop_session(id: u64) -> Result<String, String> {
    let sessions = sessions().lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get(&id)
        .ok_or_else(|| "Session not found".to_string())?;

    let running = session
        .status
        .lock()
        .map(|s| *s == Status::Running)
        .unwrap_or(false);
    if !running {
        return Err("Session is not running".to_string());
    }

    if let Ok(mut s) = session.status.lock() {
        *s = Status::Stopped;
    }
    let pid = session.pid.load(Ordering::Relaxed);
    kill_pid(pid);
    append_output(session, "\n[stopped]\n");
    Ok("Execution stopped".to_string())
}

pub fn stop_all() -> Result<String, String> {
    let sessions = sessions().lock().map_err(|e| e.to_string())?;
    let mut n = 0usize;
    for session in sessions.values() {
        let running = session
            .status
            .lock()
            .map(|s| *s == Status::Running)
            .unwrap_or(false);
        if running {
            if let Ok(mut s) = session.status.lock() {
                *s = Status::Stopped;
            }
            kill_pid(session.pid.load(Ordering::Relaxed));
            append_output(session, "\n[stopped]\n");
            if let Ok(mut out) = session.output.lock() {
                out.shrink_to_fit();
            }
            n += 1;
        }
    }
    Ok(format!("Stopped {} session(s)", n))
}

pub fn clear_session(id: u64) -> Result<String, String> {
    let mut sessions = sessions().lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get(&id)
        .ok_or_else(|| "Session not found".to_string())?;
    let running = session
        .status
        .lock()
        .map(|s| *s == Status::Running)
        .unwrap_or(false);
    if running {
        return Err("Cannot clear a running session".to_string());
    }
    sessions.remove(&id);
    Ok("Session cleared".to_string())
}

pub fn clear_finished() -> Result<String, String> {
    let mut sessions = sessions().lock().map_err(|e| e.to_string())?;
    let finished: Vec<u64> = sessions
        .iter()
        .filter_map(|(id, s)| {
            let st = s.status.lock().ok()?;
            if *st != Status::Running {
                Some(*id)
            } else {
                None
            }
        })
        .collect();
    let n = finished.len();
    for id in finished {
        if let Some(session) = sessions.remove(&id) {
            if let Ok(mut out) = session.output.lock() {
                out.clear();
                out.shrink_to(0);
            }
        }
    }
    sessions.shrink_to_fit();
    Ok(format!("Cleared {} session(s)", n))
}

/// Backward-compatible single-shot wait (not used by multi-terminal UI).
#[allow(dead_code)]
pub fn execute_file(path: &Path) -> Result<String, String> {
    let info = start_execution(path)?;
    let id = info.id;
    let start = Instant::now();
    let timeout = Duration::from_secs(DEFAULT_TIMEOUT_SECS);

    loop {
        let session = get_session(id)?;
        if session.status != "running" {
            if session.status == "done" {
                return Ok(session.output);
            }
            return Err(if session.output.is_empty() {
                format!("Process ended with status: {}", session.status)
            } else {
                session.output
            });
        }
        if start.elapsed() > timeout {
            let _ = stop_session(id);
            return Err("Process timed out".to_string());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[allow(dead_code)]
pub fn stop_execution() -> Result<String, String> {
    stop_all()
}

// silence unused import warning on non-windows if Read used only partially
#[allow(dead_code)]
fn _read_hint(_: &dyn Read) {}
