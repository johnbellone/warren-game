//! Playable character: the "maga_block" voxel model with 7 animation clips.
//!
//! Loaded as a glTF scene with an `AnimationGraph` (one node per clip).
//!
//! The character is an **Avian dynamic rigid body**: gravity pulls it down and
//! a static collider slab under the voxel ground (the "voxel-top floor") holds
//! it. Horizontal WASD movement routes through `LinearVelocity` (see
//! `physics::apply_character_input`); the camera follows the physics-driven
//! position via `sync_character_state`.
//!
//! Controls:
//!   – WASD   : move the character on the XZ plane (auto-plays Walk / Idle)
//!   – Q / E  : nudge vertical velocity (does not trigger Walk)
//!   – 1–7    : manually select and lock a clip (Walk, Run, Jump, …)
//!   – M      : return to auto Walk/Idle control
//!   – T      : toggle turntable spin
//!
//! The main camera follows the character from slightly behind and above.

use bevy::prelude::*;
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex};
use bevy::animation::AnimationPlayer;
use bevy::world_serialization::{WorldAssetRoot, WorldInstanceReady};

use crate::physics::character_body_components;

const MODEL: &str = "models/maga_block.glb";
/// Clip order inside the .glb (`GltfAssetLabel::Animation` index).
const CLIPS: [&str; 7] = ["Idle", "Walk", "Run", "Sprint", "Jump", "Wave", "Cheer"];

// --------------------------------------------------------------------------
// Components & Resources
// --------------------------------------------------------------------------

/// Marker for the character root entity.
#[derive(Component)]
pub struct Character;

/// Marker for the camera that should follow the character.
#[derive(Component)]
pub struct CameraFollow;

/// Current character movement state.
#[derive(Resource, Default)]
pub struct CharacterState {
    /// Whether the turntable spin is active.
    pub spinning: bool,
    /// True while WASD is being held (used to auto-play the Walk clip).
    pub moving: bool,
    /// When true, a manually selected clip (1–7) is held; auto walk/idle is off.
    pub manual: bool,
    /// Index of the last clip auto-played (0 = Idle, 1 = Walk) to avoid
    /// re-triggering play() every frame.
    pub last_auto: usize,
    /// Last-known character world position (written by move_character,
    /// read by camera_follow_character).
    pub position: Vec3,
}

/// Tracks orbit camera state: azimuthal angle and zoom distance.
#[derive(Resource)]
pub struct CameraOrbitState {
    /// Y-axis orbit angle in radians (0 = behind character).
    pub angle: f32,
    /// Distance from character (camera zoom).
    pub distance: f32,
    /// Height above the character (constant vertical offset).
    pub height: f32,
}

impl Default for CameraOrbitState {
    fn default() -> Self {
        Self { angle: 0.0, distance: 5.0, height: 2.5 }
    }
}

/// Holds the `AnimationGraph` and its node indices for runtime switching.
#[derive(Resource, Default)]
pub struct CharacterClips {
    pub graph: Handle<AnimationGraph>,
    pub nodes: Vec<AnimationNodeIndex>,
}

// --------------------------------------------------------------------------
// Camera Follow Component
// --------------------------------------------------------------------------

/// Spawns a dedicated camera that follows the character (runs in Startup).
pub fn spawn_follower_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Initial position: 5 m behind and 2.5 m above the character at origin
        Transform::from_xyz(0.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraFollow,
    ));
}

// --------------------------------------------------------------------------
// Startup
// --------------------------------------------------------------------------

/// Spawns the character model and its animation graph on the first frame.
pub fn spawn_character(
    mut commands: Commands,
    existing: Query<&Character>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut clips_res: ResMut<CharacterClips>,
) {
    // Query<&T> matches only entities that have T, so is_empty() is true at startup.
    if !existing.is_empty() {
        return;
    }

    // One graph node per clip.
    let clip_loads: Vec<_> =
        (0..CLIPS.len()).map(|i| asset_server.load(GltfAssetLabel::Animation(i).from_asset(MODEL))).collect();
    let (graph, nodes) = AnimationGraph::from_clips(clip_loads);

    let graph_handle = graphs.add(graph);
    clips_res.graph = graph_handle.clone();
    clips_res.nodes = nodes;

    // Start 3 m up so the dynamic body falls onto the voxel-top floor and
    // settles — the character has real physics, not a static transform.
    commands
        .spawn((
            Character,
            Transform::from_xyz(0.0, 3.0, 0.0),
            character_body_components(),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
        ))
        .observe(start_character_animation);

    info!(
        "character physics body spawned: dynamic RigidBody with ~1.8m cuboid collider + voxel-top floor"
    );
}

/// Once the glTF scene has spawned, hook its `AnimationPlayer` to the graph.
fn start_character_animation(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    clips: Res<CharacterClips>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if clips.nodes.is_empty() {
        return;
    }
    let count = children.iter_descendants(ready.entity).count();
    info!("Character instance ready: {} descendant entities", count);
    for entity in children.iter_descendants(ready.entity) {
        if let Ok(mut player) = players.get_mut(entity) {
            player.play(clips.nodes[0]).repeat();
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(clips.graph.clone()));
        }
    }
}

// --------------------------------------------------------------------------
// Update Systems
// --------------------------------------------------------------------------

/// Switches the active animation clip on keypress (keys 1–7).
///
/// Selecting a clip with 1–7 locks it (manual mode) so the walk/idle auto-
/// switcher doesn't override it. Press **M** to return to auto walk/idle.
pub fn switch_animation(
    keys: Res<ButtonInput<KeyCode>>,
    clips: Res<CharacterClips>,
    mut state: ResMut<CharacterState>,
    mut players: Query<&mut AnimationPlayer>,
) {
    const KEYS: [KeyCode; 7] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
    ];
    for (i, key) in KEYS.iter().enumerate() {
        if keys.just_pressed(*key) {
            if let Some(node) = clips.nodes.get(i) {
                state.manual = true;
                for mut player in &mut players {
                    player.stop_all();
                    player.play(*node).repeat();
                }
            }
            info!("Playing animation: {}", CLIPS[i]);
        }
    }

    // M toggles back to auto walk/idle control.
    if keys.just_pressed(KeyCode::KeyM) {
        state.manual = false;
        info!("Animation control: auto (walk/idle)");
    }
}

/// Auto-switches between Idle and Walk based on whether the character is moving.
/// Only replays a clip when the target changes, so it doesn't restart every
/// frame. Skipped while a manual clip is selected (press M to resume auto).
pub fn auto_animation(
    clips: Res<CharacterClips>,
    mut state: ResMut<CharacterState>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if state.manual || clips.nodes.is_empty() {
        return;
    }

    let target = if state.moving { 1 } else { 0 }; // 1 = Walk, 0 = Idle
    if target == state.last_auto {
        return;
    }
    if let Some(node) = clips.nodes.get(target) {
        for mut player in &mut players {
            player.stop_all();
            player.play(*node).repeat();
        }
        state.last_auto = target;
    }
}

/// Reads WASD/Q/E input and delegates the character's horizontal velocity to
/// the physics module (see `physics::apply_character_input`). The character's
/// position is now owned by the physics solver, so this no longer mutates the
/// body `Transform` directly. Kept for the `state.moving`/turntable flag and
/// for the `auto_animation` system that reads it. (WASD input itself is
/// consumed by `apply_character_input`; this function is retained as a
/// placeholder and is harmless.)
#[allow(dead_code)]
pub fn move_character(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CharacterState>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        state.spinning = !state.spinning;
    }
    let _ = time;
}

/// ArrowLeft / ArrowRight orbit the camera around the character.
/// ArrowUp zooms in; ArrowDown zooms out.
pub fn camera_orbit(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut orbit: ResMut<CameraOrbitState>,
) {
    let turn_speed = 1.5 * time.delta_secs();
    let zoom_speed = 20.0 * time.delta_secs();

    if keys.pressed(KeyCode::ArrowLeft) {
        orbit.angle -= turn_speed;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        orbit.angle += turn_speed;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        orbit.distance = (orbit.distance - zoom_speed).clamp(1.5, 180.0);
    }
    if keys.pressed(KeyCode::ArrowDown) {
        orbit.distance = (orbit.distance + zoom_speed).clamp(1.5, 180.0);
    }
}

/// Smoothly follows the character using orbit state (angle + distance) and CharacterState.
pub fn camera_follow_character(
    time: Res<Time>,
    state: Res<CharacterState>,
    orbit: Res<CameraOrbitState>,
    mut cam_q: Query<&mut Transform, With<CameraFollow>>,
) {
    // Camera offset from character in spherical coordinates
    let dx = orbit.angle.cos() * orbit.distance;
    let dz = orbit.angle.sin() * orbit.distance;
    let target = Vec3::new(
        state.position.x + dx,
        state.position.y + orbit.height,
        state.position.z + dz,
    );
    for mut cam_t in &mut cam_q {
        cam_t.translation = cam_t.translation.lerp(target, 5.0 * time.delta_secs());
        cam_t.look_at(
            Vec3::new(state.position.x, state.position.y + 0.9, state.position.z),
            Vec3::Y,
        );
    }
}
