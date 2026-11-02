// Warren Game Editor — Tauri desktop client
use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri::command;

// ── State ────────────────────────────────────────────────────────────────────

/// Holds the running Bevy engine process handle.
#[derive(Default)]
struct EngineState {
    child: Option<Child>,
}

/// Holds the running server process handle.
#[derive(Default)]
struct ServerState {
    child: Option<Child>,
    addr: String,
}

/// Holds the editor state (brush colour, tool mode) shared between Tauri and UI.
#[derive(Clone, Default)]
struct EditorState {
    color: [f32; 3],
    tool: String,
}

// ── Engine lifecycle ─────────────────────────────────────────────────────────

#[command]
fn start_engine(state: State<Mutex<EngineState>>) -> Result<String, String> {
    let mut guard = state.inner().lock().unwrap();
    log::info!("[IPC] start_engine called");
    if guard.child.is_some() {
        return Ok("engine already running".to_string());
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let binary = std::path::Path::new(&manifest_dir)
        .join("..").join("target").join("debug").join("warren-game-app");
    if !binary.exists() {
        return Err(format!("binary not found at {}", binary.display()));
    }
    let child = Command::new(&binary).spawn()
        .map_err(|e| format!("spawn error: {e}"))?;
    let pid = child.id();
    log::info!("[IPC] spawned warren-game-app PID {pid}");
    *guard = EngineState { child: Some(child) };
    Ok(format!("engine started (PID {pid})"))
}

#[command]
fn stop_engine(state: State<Mutex<EngineState>>) -> String {
    log::info!("[IPC] stop_engine called");
    let mut guard = state.inner().lock().unwrap();
    match guard.child.take() {
        Some(mut child) => {
            let pid = child.id();
            let _ = child.kill();
            log::info!("[IPC] killed warren-game PID {pid}");
            format!("engine stopped (PID {pid})")
        }
        None => "no engine to stop".to_string(),
    }
}

// ── Server lifecycle ─────────────────────────────────────────────────────────

#[command]
fn start_server(state: State<Mutex<ServerState>>) -> Result<String, String> {
    log::info!("[IPC] start_server called");
    let mut guard = state.inner().lock().unwrap();
    if guard.child.is_some() {
        return Ok(format!("server already running on {}", guard.addr));
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let binary = std::path::Path::new(&manifest_dir)
        .join("..").join("target").join("debug").join("warren-server");
    if !binary.exists() {
        return Err(format!("binary not found at {}", binary.display()));
    }
    let child = Command::new(&binary)
        .args(["--port", "8080"])
        .spawn()
        .map_err(|e| format!("spawn error: {e}"))?;
    let pid = child.id();
    let addr = "ws://127.0.0.1:8080".to_string();
    log::info!("[IPC] spawned warren-server PID {pid}");
    *guard = ServerState { child: Some(child), addr };
    Ok("server started on ws://127.0.0.1:8080".to_string())
}

#[command]
fn stop_server(state: State<Mutex<ServerState>>) -> String {
    log::info!("[IPC] stop_server called");
    let mut guard = state.inner().lock().unwrap();
    match guard.child.take() {
        Some(mut child) => {
            let pid = child.id();
            let _ = child.kill();
            log::info!("[IPC] killed warren-server PID {pid}");
            format!("server stopped (PID {pid})")
        }
        None => "no server to stop".to_string(),
    }
}

// ── Editor state (colour, tool) ──────────────────────────────────────────────

#[command]
fn set_color(state: State<Mutex<EditorState>>, r: f32, g: f32, b: f32) -> String {
    log::info!("[IPC] set_color r={r} g={g} b={b}");
    let mut guard = state.inner().lock().unwrap();
    guard.color = [r, g, b];
    format!("color set to [{r}, {g}, {b}]")
}

#[command]
fn get_color(state: State<Mutex<EditorState>>) -> [f32; 3] {
    let guard = state.inner().lock().unwrap();
    guard.color
}

#[command]
fn set_tool(state: State<Mutex<EditorState>>, tool: String) -> String {
    log::info!("[IPC] set_tool tool={tool}");
    let mut guard = state.inner().lock().unwrap();
    guard.tool = tool.clone();
    format!("tool set to {tool}")
}

// ── Roster ───────────────────────────────────────────────────────────────────

#[command]
fn get_roster() -> String {
    log::info!("[IPC] get_roster called (MVP: returns empty roster)");
    r#"[]"#.to_string()
}

// ── Greeting (kept as smoke test) ────────────────────────────────────────────

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

// ── Tauri run ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(EngineState::default()))
        .manage(Mutex::new(ServerState::default()))
        .manage(Mutex::new(EditorState::default()))
        .setup(|app| {
            log::info!("[TAURI] app started — Mantine UI webview + Bevy engine ready");
            // IPC round-trip: call a registered Tauri command function and log the result
            let response = greet("tauri-smoke-test");
            log::info!("[IPC] greet round-trip → {response}");
            let color = app.state::<Mutex<EditorState>>().lock().unwrap().color;
            log::info!("[IPC] editor default color: {color:?}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_engine, stop_engine,
            start_server, stop_server,
            set_color, get_color, set_tool,
            get_roster,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_works() {
        assert_eq!(greet("world"), "Hello, world!");
    }

    #[test]
    fn set_tool_updates_state() {
        let state = Mutex::new(EditorState::default());
        let tool = "erase";
        {
            let mut guard = state.lock().unwrap();
            guard.tool = tool.to_string();
        }
        let guard = state.lock().unwrap();
        assert_eq!(guard.tool, "erase");
    }

    #[test]
    fn set_color_updates_state() {
        let state = Mutex::new(EditorState::default());
        {
            let mut guard = state.lock().unwrap();
            guard.color = [1.0, 0.5, 0.0];
        }
        let guard = state.lock().unwrap();
        assert_eq!(guard.color, [1.0, 0.5, 0.0]);
    }
}
