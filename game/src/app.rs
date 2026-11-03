//! WarrenGamePlugin: wires up all Bevy systems.

use bevy::prelude::*;

use crate::character::{
    camera_follow_character, move_character, spawn_character, spawn_follower_camera,
    switch_animation, CharacterClips, CharacterState,
};
use crate::hotbar::{Hotbar, update_hotbar};
use crate::hotbar_hud::{spawn_hotbar_hud, update_hotbar_hud, HotbarHudState};
use crate::net::{
    handle_click_net, heartbeat, init_net_connection, process_inbox, send_queued_edits,
    ClientConnection, HeartbeatTimer, OutgoingEdits,
};
use crate::rendering::{
    move_camera, setup_camera, setup_environment, sync_voxels_to_scene, VoxelWorldResource,
};

/// The main game plugin.
pub struct WarrenGamePlugin;

impl Plugin for WarrenGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorldResource>()
            .init_resource::<ClientConnection>()
            .init_resource::<OutgoingEdits>()
            .init_resource::<HeartbeatTimer>()
            .init_resource::<Hotbar>()
            .init_resource::<HotbarHudState>()
            .init_resource::<CharacterState>()
            .init_resource::<CharacterClips>()
            .add_systems(
                Startup,
                (
                    setup_environment,
                    setup_camera,
                    spawn_follower_camera,
                    spawn_character,
                    spawn_hotbar_hud,
                    init_net_connection,
                ),
            )
            .add_systems(
                Update,
                move_character,
            )
            .add_systems(
                Update,
                camera_follow_character.after(move_character),
            )
            .add_systems(
                Update,
                (
                    switch_animation,
                    update_hotbar,
                    move_camera,
                    handle_click_net,
                    send_queued_edits,
                    process_inbox,
                    heartbeat,
                    sync_voxels_to_scene,
                    update_hotbar_hud,
                ),
            );
    }
}
