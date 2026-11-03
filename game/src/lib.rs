//! Warren Game — Bevy Voxel Editor
//!
//! Phases 3–5: rendering, networking, hotbar, HUD, and playable character.

pub mod world;
pub mod rendering;
pub mod hotbar;
pub mod hotbar_hud;
pub mod character;
pub mod net;
pub mod app;

pub use app::WarrenGamePlugin;

pub use net::{
    init_net_connection, heartbeat, process_inbox, send_queued_edits, ClientConnection,
};

pub use hotbar::{update_hotbar, BlockPreset, Hotbar};

pub use hotbar_hud::{update_hotbar_hud, HotbarHudState, spawn_hotbar_hud};

pub use character::{
    camera_follow_character, move_character, spawn_character, spawn_follower_camera,
    switch_animation, CameraFollow, Character, CharacterClips, CharacterState,
};
