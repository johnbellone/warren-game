//! WarrenGamePlugin: wires up all Bevy systems.

use bevy::prelude::*;

use crate::character::{
    camera_follow_character, camera_orbit, move_character, spawn_character, spawn_follower_camera,
    switch_animation, auto_animation, CameraOrbitState, CharacterClips, CharacterState,
};
use crate::hotbar::{Hotbar, update_hotbar};
use crate::net::{
    handle_click_net, heartbeat, init_net_connection, process_inbox, send_queued_edits,
    ClientConnection, HeartbeatTimer, OutgoingEdits,
};
use crate::neighborhood::load_capitol_neighborhood;
use crate::rendering::{setup_environment, sync_voxels_to_scene, VoxelWorldResource};

/// The main game plugin.
pub struct WarrenGamePlugin;

impl Plugin for WarrenGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorldResource>()
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
                    move_character,
                    camera_orbit,
                    camera_follow_character,
                    switch_animation,
                    auto_animation,
                    update_hotbar,
                    handle_click_net,
                    send_queued_edits,
                    process_inbox,
                    heartbeat,
                    sync_voxels_to_scene,
                ),
            );
    }
}
