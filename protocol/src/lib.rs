//! Warren Game Protocol — shared types for client/server communication.
//! All types here are serializable and form the contract between the
//! WebSocket server and Bevy clients.

use serde::{Deserialize, Serialize};

/// The type of asset being edited.
/// In MVP only Voxel is fully implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    /// A single cube in the voxel world.
    Voxel,
    /// A reference to a 3D model (OBJ/GLTF). Not implemented in MVP.
    Model,
    /// PBR material properties. Not implemented in MVP.
    Material,
    /// A physics property attached to an asset. Not implemented in MVP.
    PhysicsProp,
    /// A script block. Not implemented in MVP.
    Script,
}

/// A single voxel in 3D space. A unit cube at `position` with `color`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Voxel {
    /// World-space position of the cube (center).
    pub position: [i32; 3],
    /// RGB color as floats 0.0..1.0.
    pub color: [f32; 3],
    /// The asset type. Always `Voxel` in MVP.
    pub asset_type: AssetType,
}

/// A voxel edit delta. Sent on a per-voxel level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoxelEdit {
    /// World-space position of the edited cube.
    pub position: [i32; 3],
    /// New color of the cube.
    pub color: [f32; 3],
    /// `true` means "erase this voxel"; `false` means "set this voxel to `color`".
    pub erase: bool,
}

/// Information about a connected peer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Unique client identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Avatar color, RGB 0.0..1.0.
    pub color_rgb: [f32; 3],
    /// Last activity time in milliseconds since epoch.
    pub last_active_ms: u64,
}

// --------------------------------------------------------------------------
// Client → Server messages
// --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    /// Announce presence to the server.
    Join {
        name: String,
        color_rgb: [f32; 3],
    },
    /// A local voxel edit. Server validates and broadcasts to all other clients.
    VoxelEdit {
        position: [i32; 3],
        color: [f32; 3],
        erase: bool,
    },
    /// Keep-alive heartbeat.
    Heartbeat,
    /// Graceful disconnect. Server removes the peer and broadcasts an updated roster.
    Leave,
}

// --------------------------------------------------------------------------
// Server → Client messages
// --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMsg {
    /// Sent once on connect. Contains the full voxel snapshot and current roster.
    Welcome {
        client_id: String,
        snapshot: Vec<Voxel>,
        roster: Vec<PeerInfo>,
    },
    /// Broadcast when a peer joins or leaves.
    Roster {
        users: Vec<PeerInfo>,
    },
    /// Broadcast for each voxel edit from any client.
    VoxelEdit {
        client_id: String,
        position: [i32; 3],
        color: [f32; 3],
        erase: bool,
    },
    /// Acknowledges a client heartbeat.
    HeartbeatAck,
    /// An error message from the server.
    Error {
        message: String,
    },
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_edit_serialization() {
        let edit = VoxelEdit {
            position: [0, 0, 0],
            color: [1.0, 0.0, 0.0],
            erase: false,
        };
        let json = serde_json::to_string(&edit).unwrap();
        let de: VoxelEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(de.position, [0, 0, 0]);
        assert_eq!(de.color, [1.0, 0.0, 0.0]);
        assert!(!de.erase);
    }

    #[test]
    fn test_client_msg_join() {
        let msg = ClientMsg::Join {
            name: "Alice".to_string(),
            color_rgb: [1.0, 0.5, 0.0],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let de: ClientMsg = serde_json::from_str(&json).unwrap();
        match de {
            ClientMsg::Join { name, .. } => assert_eq!(name, "Alice"),
            other => panic!("expected Join, got {:?}", other),
        }
    }

    #[test]
    fn test_server_msg_welcome() {
        let msg = ServerMsg::Welcome {
            client_id: "abc123".to_string(),
            snapshot: vec![
                Voxel {
                    position: [0, 0, 0],
                    color: [1.0, 0.0, 0.0],
                    asset_type: AssetType::Voxel,
                },
            ],
            roster: vec![
                PeerInfo {
                    id: "abc123".to_string(),
                    name: "Alice".to_string(),
                    color_rgb: [1.0, 0.0, 0.0],
                    last_active_ms: 0,
                },
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let de: ServerMsg = serde_json::from_str(&json).unwrap();
        match de {
            ServerMsg::Welcome { client_id, snapshot, roster, .. } => {
                assert_eq!(client_id, "abc123");
                assert_eq!(snapshot.len(), 1);
                assert_eq!(roster.len(), 1);
            }
            other => panic!("expected Welcome, got {:?}", other),
        }
    }

    #[test]
    fn test_server_msg_voxel_edit() {
        let msg = ServerMsg::VoxelEdit {
            client_id: "client1".to_string(),
            position: [1, 2, 3],
            color: [0.0, 1.0, 0.0],
            erase: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let de: ServerMsg = serde_json::from_str(&json).unwrap();
        match de {
            ServerMsg::VoxelEdit { client_id, position, color, erase, .. } => {
                assert_eq!(client_id, "client1");
                assert_eq!(position, [1, 2, 3]);
                assert_eq!(color, [0.0, 1.0, 0.0]);
                assert!(!erase);
            }
            other => panic!("expected VoxelEdit, got {:?}", other),
        }
    }
}
