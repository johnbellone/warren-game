//! Core server logic: state management, voxel world, and WebSocket handling.
//!
//! The `ServerState` type is the canonical shared state. It uses `broadcast`
//! to fan-out JSON string messages to all connected clients.
//!
//! This module is the reusable part — it can be embedded in Tauri or run
//! standalone via the `warren-server` binary.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use log::{info, warn};
use warren_protocol::{ClientMsg, PeerInfo, ServerMsg, Voxel};

/// The canonical shared server state.
/// Uses `tokio::sync::Mutex` + `tokio::sync::broadcast` for multi-client coordination.
#[derive(Clone)]
pub struct ServerState {
    /// Current voxels in the shared world.
    pub voxels: HashMap<[i32; 3], Voxel>,
    /// Connected peers keyed by client ID.
    pub peers: HashMap<String, PeerInfo>,
    /// Broadcast channel for JSON string messages to all clients.
    pub broadcast: tokio::sync::broadcast::Sender<String>,
    /// Counter for client IDs.
    pub next_id: u32,
}

impl ServerState {
    /// Create a new server state with an empty world and a fresh broadcast channel.
    pub fn new(broadcast_capacity: usize) -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel(broadcast_capacity);
        Self {
            voxels: HashMap::new(),
            peers: HashMap::new(),
            broadcast: tx,
            next_id: 1,
        }
    }

    /// Add a new peer and broadcast the welcome + updated roster to all.
    pub fn add_peer(&mut self, client_id: String, name: String, color_rgb: [f32; 3]) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.peers.insert(
            client_id.clone(),
            PeerInfo {
                id: client_id.clone(),
                name: name.clone(),
                color_rgb,
                last_active_ms: now_ms,
            },
        );

        let roster: Vec<PeerInfo> = self.peers.values().cloned().collect();

        // Send Welcome to this peer with full snapshot
        let snapshot: Vec<Voxel> = self.voxels.values().cloned().collect();
        let welcome = ServerMsg::Welcome {
            client_id: client_id.clone(),
            snapshot,
            roster: roster.clone(),
        };
        if let Ok(json) = serde_json::to_string(&welcome) {
            let _ = self.broadcast.send(json);
        }

        // Broadcast roster to all
        let roster_msg = ServerMsg::Roster { users: roster };
        if let Ok(json) = serde_json::to_string(&roster_msg) {
            let _ = self.broadcast.send(json);
        }
    }

    /// Remove a peer by ID and broadcast the updated roster.
    pub fn remove_peer(&mut self, client_id: &str) {
        if self.peers.remove(client_id).is_some() {
            let roster: Vec<PeerInfo> = self.peers.values().cloned().collect();
            let roster_msg = ServerMsg::Roster { users: roster };
            if let Ok(json) = serde_json::to_string(&roster_msg) {
                let _ = self.broadcast.send(json);
            }
        }
    }

    /// Apply a voxel edit (place or erase) and broadcast to all clients.
    pub fn apply_voxel_edit(
        &mut self,
        client_id: &str,
        position: [i32; 3],
        color: [f32; 3],
        erase: bool,
    ) {
        if erase {
            self.voxels.remove(&position);
        } else {
            self.voxels.insert(
                position,
                Voxel {
                    position,
                    color,
                    asset_type: warren_protocol::AssetType::Voxel,
                },
            );
        }

        let edit = ServerMsg::VoxelEdit {
            client_id: client_id.to_string(),
            position,
            color,
            erase,
        };
        if let Ok(json) = serde_json::to_string(&edit) {
            let _ = self.broadcast.send(json);
        }
    }

    /// Process a client message and apply it to the world.
    pub fn process_message(&mut self, client_id: &str, msg: &ClientMsg) {
        match msg {
            ClientMsg::Join { name, color_rgb } => {
                info!("Client {} joined as '{}'", client_id, name);
                self.add_peer(client_id.to_string(), name.clone(), *color_rgb);
            }
            ClientMsg::VoxelEdit { position, color, erase } => {
                info!(
                    "Client {} voxel {} at {:?}",
                    client_id,
                    if *erase { "ERASE" } else { "PLACE" },
                    position
                );
                self.apply_voxel_edit(client_id, *position, *color, *erase);
            }
            ClientMsg::Heartbeat => {
                // No broadcast, just log if needed
            }
            ClientMsg::Leave => {
                info!("Client {} sent Leave", client_id);
            }
        }
    }
}

/// Handle a single WebSocket connection. Called from `main.rs` via `tokio::spawn`.
#[allow(dead_code)]
pub async fn handle_client(
    state: Arc<tokio::sync::Mutex<ServerState>>,
    mut ws: WebSocketStream<TcpStream>,
    client_id: String,
    mut broadcast_rx: tokio::sync::broadcast::Receiver<String>,
) {
    info!("Client {} connected", client_id);

    loop {
        tokio::select! {
            result = ws.next() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(msg) = serde_json::from_str::<ClientMsg>(&text) {
                            state.lock().await.process_message(&client_id, &msg);
                        } else {
                            warn!("Client {} sent unparseable text: {}", client_id, text);
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Client {} disconnected", client_id);
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        warn!("Client {} websocket error: {}", client_id, e);
                        break;
                    }
                    None => break,
                }
            }

            result = broadcast_rx.recv() => {
                match result {
                    Ok(text) => {
                        if ws.send(Message::Text(text)).await.is_err() {
                            info!("Client {} send failed, disconnecting", client_id);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Client {} broadcast reader lagged by {} msgs", client_id, n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    state.lock().await.remove_peer(&client_id);
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_peer() {
        let mut state = ServerState::new(1024);
        state.add_peer("c1".to_string(), "Alice".to_string(), [1.0, 0.0, 0.0]);
        assert_eq!(state.peers.len(), 1);
        state.remove_peer("c1");
        assert_eq!(state.peers.len(), 0);
    }

    #[test]
    fn test_voxel_place_and_erase() {
        let mut state = ServerState::new(1024);
        state.apply_voxel_edit("c1", [0, 0, 0], [1.0, 0.0, 0.0], false);
        assert_eq!(state.voxels.len(), 1);
        assert!(state.voxels.contains_key(&[0, 0, 0]));

        state.apply_voxel_edit("c1", [0, 0, 0], [0.0, 0.0, 0.0], true);
        assert_eq!(state.voxels.len(), 0);
    }

    #[test]
    fn test_multiple_peers() {
        let mut state = ServerState::new(1024);
        state.add_peer("c1".to_string(), "Alice".to_string(), [1.0, 0.0, 0.0]);
        state.add_peer("c2".to_string(), "Bob".to_string(), [0.0, 1.0, 0.0]);
        state.add_peer("c3".to_string(), "Carol".to_string(), [0.0, 0.0, 1.0]);
        assert_eq!(state.peers.len(), 3);
    }

    #[test]
    fn test_process_join_message() {
        let mut state = ServerState::new(1024);
        let msg = ClientMsg::Join {
            name: "Alice".to_string(),
            color_rgb: [1.0, 0.0, 0.0],
        };
        state.process_message("c1", &msg);
        assert_eq!(state.peers.len(), 1);
        assert!(state.peers.contains_key("c1"));
    }

    #[test]
    fn test_process_voxel_edit() {
        let mut state = ServerState::new(1024);
        let msg = ClientMsg::VoxelEdit {
            position: [5, 3, 2],
            color: [0.0, 1.0, 0.0],
            erase: false,
        };
        state.process_message("c1", &msg);
        assert_eq!(state.voxels.len(), 1);
        assert!(state.voxels.contains_key(&[5, 3, 2]));
    }
}
