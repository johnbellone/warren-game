//! WarrenGamePlugin: wires up all Bevy systems.

use bevy::prelude::*;

use crate::hotbar::{Hotbar, update_hotbar};
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
            .add_systems(
                Startup,
                (
                    setup_environment,
                    setup_camera,
                    init_net_connection,
                ),
            )
            .add_systems(
                Update,
                (
                    update_hotbar,   // hotbar selection first
                    move_camera,
                    handle_click_net, // uses Hotbar for brush_color
                    send_queued_edits,
                    process_inbox,
                    heartbeat,
                    sync_voxels_to_scene,
                ),
            );
    }
}
