//! Playable character: the "maga_block" voxel model with 7 animation clips.
//!
//! Loaded as a glTF scene with an `AnimationGraph` (one node per clip).
//!
//! Controls:
//!   – WASD   : move character on the XZ plane
//!   – Q / E  : adjust height (y-axis)
//!   – 1–7    : switch animation clip
//!   – T      : toggle turntable spin
//!
//! The main camera follows the character from slightly behind and above.

use bevy::prelude::*;
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex};
use bevy::animation::AnimationPlayer;
use bevy::world_serialization::{WorldAssetRoot, WorldInstanceReady};

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
    /// Last-known character world position (written by move_character, read by camera_follow_character).
    pub position: Vec3,
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
        Transform::from_xyz(0.0, 3.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraFollow,
    ));
}

// --------------------------------------------------------------------------
// Startup
// --------------------------------------------------------------------------

/// Spawns the character model and its animation graph on the first frame.
pub fn spawn_character(
    mut commands: Commands,
    existing: Query<Option<&Character>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut clips_res: ResMut<CharacterClips>,
) {
    // Guard: don't spawn twice.
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

    commands
        .spawn((
            Character,
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
        ))
        .observe(start_character_animation);
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
pub fn switch_animation(
    keys: Res<ButtonInput<KeyCode>>,
    clips: Res<CharacterClips>,
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
                for mut player in &mut players {
                    player.stop_all();
                    player.play(*node).repeat();
                }
            }
            info!("Playing animation: {}", CLIPS[i]);
        }
    }
}

/// Moves the character via WASD (XZ) and Q/E (height); T toggles turntable.
/// Also writes the character position to CharacterState for the camera follow system.
pub fn move_character(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CharacterState>,
    mut chars: Query<&mut Transform, With<Character>>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        state.spinning = !state.spinning;
    }

    let speed = 3.0 * time.delta_secs();
    let mut move_dir = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) { move_dir.z += 1.0; }
    if keys.pressed(KeyCode::KeyS) { move_dir.z -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }
    if keys.pressed(KeyCode::KeyQ) { move_dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyE) { move_dir.y += 1.0; }

    for mut t in &mut chars {
        if move_dir != Vec3::ZERO {
            t.translation += move_dir * speed;
        }
        if state.spinning {
            t.rotate_y(speed * 0.8);
        }
        // Cache position for the camera follow system (avoids &mut -> & read conflict on Transform)
        state.position = t.translation;
    }
}

/// Smoothly follows the character using the position read from CharacterState (no Transform conflict).
pub fn camera_follow_character(
    time: Res<Time>,
    state: Res<CharacterState>,
    mut cam_q: Query<&mut Transform, With<CameraFollow>>,
) {
    let target = state.position + Vec3::new(0.0, 2.5, 4.0);
    for mut cam_t in &mut cam_q {
        cam_t.translation = cam_t.translation.lerp(target, 3.0 * time.delta_secs());
        cam_t.look_at(state.position, Vec3::Y);
    }
}
