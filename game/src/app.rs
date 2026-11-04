//! WarrenGamePlugin: wires up all Bevy systems.

use bevy::prelude::*;
use avian3d::prelude as avian;

use crate::character::{
    camera_follow_character, camera_orbit, spawn_character, spawn_follower_camera,
    switch_animation, auto_animation, CameraOrbitState, CharacterClips, CharacterState,
};
use crate::hotbar::{Hotbar, update_hotbar};
use crate::net::{
    handle_click_net, heartbeat, init_net_connection, process_inbox, send_queued_edits,
    ClientConnection, HeartbeatTimer, OutgoingEdits,
};
use crate::neighborhood::load_capitol_neighborhood;
use crate::physics::{apply_character_input, setup_physics, sync_character_state};
use crate::rendering::{setup_environment, sync_chunk_meshes, VoxelWorldResource};

/// The main game plugin.
pub struct WarrenGamePlugin;

impl Plugin for WarrenGamePlugin {
    fn build(&self, app: &mut App) {
        // --- Physics (Avian 3D XPBD) ---------------------------------------
        // The character is a dynamic rigid body, so Avian must be installed
        // first. `setup_physics` seeds a downward Gravity + the static voxel-top
        // floor collider so the body falls and rests on the ground.
        app.add_plugins(avian::PhysicsPlugins::default())
            .add_systems(Startup, setup_physics)
            .init_resource::<VoxelWorldResource>()
            .init_resource::<ClientConnection>()
            .init_resource::<OutgoingEdits>()
            .init_resource::<HeartbeatTimer>()
            .init_resource::<Hotbar>()
            .init_resource::<CharacterState>()
            .init_resource::<CharacterClips>()
            .init_resource::<CameraOrbitState>()
            .add_systems(
                Startup,
                (
                    setup_environment,
                    spawn_follower_camera,
                    spawn_character,
                    load_capitol_neighborhood,
                    init_net_connection,
                ),
            )
            .add_systems(
                Update,
                (
                    apply_character_input,
                    camera_orbit,
                    sync_character_state,
                    camera_follow_character,
                    switch_animation,
                    auto_animation,
                    update_hotbar,
                    handle_click_net,
                    send_queued_edits,
                    process_inbox,
                    heartbeat,
                    sync_chunk_meshes,
                ),
            );
    }
}
