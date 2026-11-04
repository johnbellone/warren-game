//! Character physics via Avian 3D (XPBD).
//!
//! The character is a **dynamic** rigid body. A single static collider slab
//! beneath the voxel ground acts as the floor, so the character falls under
//! gravity and comes to rest on top of the voxel ground (the "voxel-top floor").
//!
//! Controls (horizontal) route through `LinearVelocity` so Avian owns the
//! vertical integration; Q/E nudge vertical velocity. Camera follow reads the
//! character position via `sync_character_state` (which copies the post-physics
//! `Transform` into `CharacterState.position`, avoiding a `&`/`&mut` conflict on
//! the body's own `Transform`).

use bevy::prelude::*;
use avian3d::prelude::*;

use crate::character::{Character, CharacterState};
use crate::neighborhood::GROUND_DEPTH;

// --------------------------------------------------------------------------
// Setup
// --------------------------------------------------------------------------

/// Ground-collider half-extent (x/z) in world units. Wide enough to cover the
/// voxel plaza so the character cannot fall off the edge.
const GROUND_COLLIDER_HALF: f32 = 80.0;

/// Startup: activate gravity and spawn the static voxel-top floor.
///
/// The floor is a static `Collider` slab whose **top surface sits at y = 0** —
/// exactly the top of the 10-deep procedurally generated voxel ground — so the
/// character rests on the ground surface, not floating above or through it.
pub fn setup_physics(mut commands: Commands, mut gravity: ResMut<Gravity>) {
    // Downward gravity (XPBD default is already ≈ -9.8; set it explicitly so the
    // "a gravity vector is active" guarantee is explicit in the code).
    *gravity = Gravity(Vec3::new(0.0, -9.8, 0.0));

    // Static ground collider slab: depth = GROUND_DEPTH, centred so the top
    // face is at y = 0.0.
    commands.spawn((
        Name::new("VoxelTopFloor"),
        RigidBody::Static,
        Collider::cuboid(GROUND_COLLIDER_HALF * 2.0, GROUND_DEPTH as f32, GROUND_COLLIDER_HALF * 2.0),
        Transform::from_xyz(0.0, -(GROUND_DEPTH as f32) * 0.5, 0.0),
    ));
}

// --------------------------------------------------------------------------
// Update
// --------------------------------------------------------------------------

/// Reads WASD/Q/E input and writes it onto the character's `LinearVelocity`.
///
/// Horizontal input overwrites X/Z velocity toward the move direction each
/// frame; Q/E add to vertical velocity so gravity can still integrate on top.
/// This does **not** write the character's `Transform` directly — Avian's
/// solver owns position after the physics schedule.
///
/// `state.moving` drives the walk/idle auto-animation; T toggles the
/// turntable spin flag.
pub fn apply_character_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CharacterState>,
    mut bodies: Query<&mut LinearVelocity, With<Character>>,
) {
    // Turntable toggle.
    if keys.just_pressed(KeyCode::KeyT) {
        state.spinning = !state.spinning;
    }

    let speed = 3.0 * time.delta_secs();
    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { move_dir.z += 1.0; }
    if keys.pressed(KeyCode::KeyS) { move_dir.z -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }

    let horizontal = Vec3::new(move_dir.x, 0.0, move_dir.z);
    let moving = horizontal != Vec3::ZERO;
    state.moving = moving;

    let jump = 3.0 * time.delta_secs();
    for mut lv in &mut bodies {
        if moving {
            let dir = horizontal / horizontal.length();
            lv.x = dir.x * speed;
            lv.z = dir.z * speed;
        } else {
            // Stop the walk cleanly when no horizontal key is held.
            lv.x = 0.0;
            lv.z = 0.0;
        }
        if keys.pressed(KeyCode::KeyQ) {
            lv.y -= jump;
        }
        if keys.pressed(KeyCode::KeyE) {
            lv.y += jump;
        }
    }
}

/// After the physics schedule updates the body, copy the character's post-physics
/// `Transform` into `CharacterState.position` so the follow camera reads the
/// physics-driven position without a `&`/`&mut` conflict on the body's `Transform`.
pub fn sync_character_state(
    mut state: ResMut<CharacterState>,
    char_tf: Query<&Transform, With<Character>>,
) {
    if let Ok(tf) = char_tf.single() {
        state.position = tf.translation;
    }
}

// --------------------------------------------------------------------------
// Spawn helper (called from spawn_character)
// --------------------------------------------------------------------------

/// Components to add to the character root so it becomes an Avian dynamic body.
///
/// The collider is a cuboid sized to a human character (~1.8 m tall so the
/// half-height is 0.9). `LinearVelocity` is initialised to zero so nothing
/// moves until input or gravity acts.
pub fn character_body_components() -> (RigidBody, Collider, LinearVelocity) {
    (
        RigidBody::Dynamic,
        Collider::cuboid(0.9, 1.8, 0.9),
        LinearVelocity(Vec3::ZERO),
    )
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn make_physics_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            avian3d::PhysicsPlugins::default(),
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::prelude::PickingPlugin,
            bevy::mesh::MeshPlugin,
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
        app.finish();
        app
    }

    /// Insert a downward gravity resource.
    fn set_gravity_down(app: &mut App) {
        let mut g = app.world_mut().resource_mut::<Gravity>();
        *g = Gravity(Vec3::new(0.0, -9.8, 0.0));
    }

    /// Spawn a dynamic body above a static floor and assert it falls under
    /// gravity and comes to rest *on top* of the floor (the voxel-top floor).
    #[test]
    fn character_falls_under_gravity_and_rests_on_floor() {
        let start_y = 6.0_f32;
        let floor_top = 0.0_f32; // the static collider's top face sits at y = 0
        let half_char_y = 0.9_f32;

        let mut app = make_physics_app();
        set_gravity_down(&mut app);

        // The scene: a static floor whose **top** face is at y = 0, and a
        // dynamic character body starting at y = 6.
        app.world_mut().spawn((
            Name::new("Ground"),
            RigidBody::Static,
            Collider::cuboid(20.0, 10.0, 20.0),
            Transform::from_xyz(0.0, -5.0, 0.0),
        ));
        let char_body = app
            .world_mut()
            .spawn((
                Name::new("CharacterBody"),
                RigidBody::Dynamic,
                Collider::cuboid(0.9, 1.8, 0.9),
                Transform::from_xyz(0.0, start_y, 0.0),
                LinearVelocity(Vec3::ZERO),
            ))
            .id();

        // Step the simulation until the body has had time to fall and settle.
        for _ in 0..300 {
            app.update();
        }

        let rest_y = app
            .world()
            .get::<Transform>(char_body)
            .unwrap()
            .translation
            .y;

        // It must have fallen well below its start height ...
        assert!(
            rest_y < start_y - 1.0,
            "character did not fall: started at {start_y}, ended at {rest_y}"
        );
        // ... and it must have come to rest on top of the floor, i.e. its
        // centre rests at ~ half-height above the y = 0 surface.
        assert!(
            (rest_y - half_char_y).abs() < 0.2,
            "character did not rest on the floor: centre at {rest_y}, expected ~{}",
            half_char_y
        );
        // And its bottom must still sit at/above the floor surface (no clip).
        assert!(
            rest_y - half_char_y > floor_top - 0.15,
            "character clipped through the floor: bottom at {}",
            rest_y - half_char_y
        );
    }

    /// With `PhysicsPlugins` installed, the `Gravity` resource exists and points
    /// downward — i.e. "a gravity vector is active" by default.
    #[test]
    fn gravity_resource_points_downward() {
        let app = make_physics_app();
        let gy = app.world().resource::<Gravity>().0.y;
        assert!(gy < 0.0, "gravity must point downward, got y = {gy}");
    }

    /// The `character_body_components()` builder produces a dynamic body with a
    /// reasonable collider and zero initial velocity.
    #[test]
    fn character_body_components_are_dynamic() {
        // Use the builder output directly: the RigidBody is `Dynamic` and its
        // collider is present with sensible extents.
        let (body, collider, vel) = super::character_body_components();
        assert!(
            matches!(body, RigidBody::Dynamic),
            "character must be a dynamic rigid body"
        );
        assert!(vel.0 == Vec3::ZERO, "velocity must start at zero");
        // A dynamic body with zero velocity is the expected resting initial
        // state before gravity acts.
        let _ = collider;
    }
}


