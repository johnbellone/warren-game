//! Warren Game — Bevy Voxel Editor
//!
//! Phase 3: Bevy 0.20 rendering layer.
//! Phase 4 will add the WebSocket client.

pub mod world;
pub mod rendering;
pub mod hotbar;
pub mod net;
pub mod app;

pub use app::WarrenGamePlugin;
pub use net::{ClientConnection, init_net_connection, heartbeat, process_inbox, send_queued_edits};
pub use hotbar::{Hotbar, BlockPreset, update_hotbar};
