//! Warren Game Server - A server that supports both native and WebAssembly execution
//! with multiplayer collaboration capabilities

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

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub username: String,
    pub connected_at: u64,
    pub is_online: bool,
}

/// Asset representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAsset {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub version: u32,
    pub owner: String,
    pub last_modified: u64,
    pub is_locked: bool,
    pub locked_by: Option<String>,
}

/// Collaboration event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEvent {
    pub event_type: String,
    pub user_id: String,
    pub asset_id: String,
    pub timestamp: u64,
    pub data: Option<serde_json::Value>,
}

/// Server state for multiplayer collaboration
#[derive(Debug, Clone)]
pub struct ServerState {
    pub users: Vec<UserSession>,
    pub assets: Vec<GameAsset>,
    pub events: Vec<CollaborationEvent>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            users: vec![],
            assets: vec![],
            events: vec![],
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

/// Add a user to the session
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn add_user(user_id: &str, username: &str) -> UserSession {
    let session = UserSession {
        user_id: user_id.to_string(),
        username: username.to_string(),
        connected_at: 0, // In a real implementation, this would use SystemTime
        is_online: true,
    };
    
    #[cfg(not(target_family = "wasm"))]
    println!("User {} joined", username);
    
    session
}

/// Create a new asset
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn create_asset(user_id: &str, name: &str, content_type: &str) -> GameAsset {
    let asset = GameAsset {
        id: format!("asset_{}", rand::random::<u64>()),
        name: name.to_string(),
        content_type: content_type.to_string(),
        version: 1,
        owner: user_id.to_string(),
        last_modified: 0, // In a real implementation, this would use SystemTime
        is_locked: true,
        locked_by: Some(user_id.to_string()),
    };
    
    #[cfg(not(target_family = "wasm"))]
    println!("Asset {} created by user {}", name, user_id);
    
    asset
}

/// Lock an asset for editing
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lock_asset(asset_id: &str, user_id: &str) -> bool {
    #[cfg(not(target_family = "wasm"))]
    println!("Asset {} locked by user {}", asset_id, user_id);
    
    // In a real implementation, this would check the actual asset state
    true
}

/// Process game tick
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn process_tick(state: &ServerState) -> String {
    format!("Processed tick with {} users and {} assets", 
           state.users.len(), state.assets.len())
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
    fn test_user_session() {
        let session = add_user("user123", "test_user");
        assert_eq!(session.user_id, "user123");
        assert_eq!(session.username, "test_user");
    }

    #[test]
    fn test_game_asset() {
        let asset = create_asset("user123", "test_asset", "model");
        assert_eq!(asset.name, "test_asset");
        assert_eq!(asset.content_type, "model");
        assert!(asset.is_locked);
    }
}