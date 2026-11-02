//! Warren Game Server - A server that supports both native and WebAssembly execution

use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub is_wasm: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            is_wasm: cfg!(feature = "wasm"),
        }
    }
}

/// Server state
#[derive(Debug, Clone)]
pub struct GameState {
    pub players: Vec<String>,
    pub world_state: String,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: vec![],
            world_state: "Initial state".to_string(),
        }
    }
}

/// Initialize the server
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn init_server() {
    #[cfg(not(target_family = "wasm"))]
    println!("Starting Warren Game Server (native)");
    
    #[cfg(target_family = "wasm")]
    web_sys::console::log_1(&"Starting Warren Game Server (WASM)".into());
}

/// Get server configuration
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn get_config() -> ServerConfig {
    ServerConfig::default()
}

/// Process game tick
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn process_tick(state: &GameState) -> String {
    format!("Processed tick with {} players", state.players.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn test_game_state() {
        let state = GameState::new();
        assert_eq!(state.players.len(), 0);
        assert_eq!(state.world_state, "Initial state");
    }
}