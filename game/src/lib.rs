//! Warren Game — Bevy Voxel Editor
//!
//! Phases 3–5: rendering, networking, hotbar, HUD, and playable character.

pub mod world;
pub mod mesh;
pub mod neighborhood;
pub mod rendering;
pub mod physics;
pub mod collision;
pub mod path_trace;
pub mod editor;
pub mod hotbar;
pub mod hotbar_hud;
pub mod character;
pub mod net;
pub mod app;

pub use app::WarrenGamePlugin;

pub use rendering::{
    setup_environment, sync_chunk_meshes, VoxelWorldResource, Sky,
};

pub use net::{
    init_net_connection, heartbeat, process_inbox, send_queued_edits, ClientConnection,
};

pub use hotbar::{update_hotbar, BlockPreset, Hotbar};

pub use hotbar_hud::{update_hotbar_hud, HotbarHudState, spawn_hotbar_hud};

pub use character::{
    auto_animation, camera_follow_character, camera_orbit, move_character, spawn_character,
    spawn_follower_camera, switch_animation, CameraFollow, Character, CharacterClips,
    CharacterState, CameraOrbitState,
};

pub use physics::{
    apply_character_input, sync_character_state, setup_physics, character_body_components,
};

pub use neighborhood::{GROUND_DEPTH, GROUND_HALF, SCALE};

// Path-tracing preview pass (CPU ray-march + PPM dump).
pub use path_trace::{PathCamera, PathTraceResource, render_path_trace, start_path_trace, trace_frame, dump_ppm};

// Voxel-grid collision resolver (swept AABB vs VoxelWorld).
pub use collision::{integrate_against_grid, aabb_touches_solid};

// In-game editor controls (keyboard-driven edits to the VoxelWorld + Hotbar).
pub use editor::editor_edit;
