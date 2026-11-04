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
use crate::path_trace::{start_path_trace, render_path_trace};
use crate::editor::editor_edit;
use crate::rendering::{setup_environment, sync_chunk_meshes, VoxelWorldResource};

/// Per-frame voxel-grid collision: after Avian's XPBD solve, sweep the
/// character's bounding box against the voxel grid and correct the body so it
/// rests on top of voxels and cannot pass through walls (the grid controller).
fn update_voxel_collision(
    time: Res<Time>,
    world_res: Res<VoxelWorldResource>,
    mut character_state: ResMut<CharacterState>,
    mut bodies: Query<
        (&mut Transform, &mut avian::LinearVelocity),
        With<crate::character::Character>,
    >,
) {
    let dt = time.delta_secs();
    if let Ok((mut tr, mut lv)) = bodies.single_mut() {
        let t = tr.translation;
        let center = [t.x, t.y, t.z];
        let vel = [lv.x, lv.y, lv.z];
        let (corr_c, corr_v) =
            crate::collision::integrate_against_grid(&world_res.world, center, vel, dt);
        tr.translation = bevy::math::Vec3::new(corr_c[0], corr_c[1], corr_c[2]);
        lv.x = corr_v[0];
        lv.y = corr_v[1];
        lv.z = corr_v[2];
        character_state.position = tr.translation;
    }
}

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
                    start_path_trace,
                ),
            )
            .add_systems(
                Update,
                (
                    apply_character_input,
                    update_voxel_collision,
                    camera_orbit,
                    sync_character_state,
                    camera_follow_character,
                    switch_animation,
                    auto_animation,
                    update_hotbar,
                    editor_edit,
                    handle_click_net,
                    send_queued_edits,
                    process_inbox,
                    heartbeat,
                    sync_chunk_meshes,
                    render_path_trace,
                ),
            );
    }
}
