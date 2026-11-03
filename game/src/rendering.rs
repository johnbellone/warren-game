//! Rendering layer: VoxelWorldResource, VoxelCube component, and sync systems.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::character::CameraFollow;
use crate::world::VoxelWorld;

// --------------------------------------------------------------------------
// Components
// --------------------------------------------------------------------------

/// Marker + position for a spawned voxel cube entity.
#[derive(Component)]
pub struct VoxelCube {
    pub position: [i32; 3],
}

/// Marker component for the ground plane.
#[derive(Component)]
pub struct Ground;

// --------------------------------------------------------------------------
// Resources
// --------------------------------------------------------------------------

/// Shared voxel world data used by rendering sync and the (future) WebSocket client.
#[derive(Resource)]
pub struct VoxelWorldResource {
    pub world: VoxelWorld,
    /// Mapping of world position → spawned Bevy entity.
    pub spawned: HashMap<[i32; 3], Entity>,
    /// The shared 1×1×1 cuboid mesh handle (created once in setup_environment).
    pub cube_mesh: Handle<Mesh>,
    /// The currently selected brush colour (RGB 0.0–1.0).
    pub brush_color: [f32; 3],
}

impl Default for VoxelWorldResource {
    fn default() -> Self {
        Self {
            world: VoxelWorld::default(),
            spawned: HashMap::new(),
            cube_mesh: Handle::default(),
            brush_color: [1.0, 0.0, 0.0],
        }
    }
}

// --------------------------------------------------------------------------
// Sync system: diff VoxelWorld against spawned entities
// --------------------------------------------------------------------------

/// Spawns a `Mesh3d` for each new voxel and despawns cubes whose position
/// no longer exists in the world.
pub fn sync_voxels_to_scene(
    mut world_res: ResMut<VoxelWorldResource>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Phase 1: collect positions to remove (in `spawned` but not in `world`)
    let to_remove: Vec<[i32; 3]> = world_res
        .spawned
        .keys()
        .copied()
        .filter(|&pos| !world_res.world.voxels.contains_key(&pos))
        .collect();

    // Phase 2: despawn removed voxels
    for pos in &to_remove {
        if let Some(entity) = world_res.spawned.remove(pos) {
            commands.entity(entity).despawn();
        }
    }

    // Phase 3: collect positions to add (in `world` but not in `spawned`)
    let to_add: Vec<([i32; 3], Color)> = world_res
        .world
        .voxels
        .iter()
        .filter(|(pos, _)| !world_res.spawned.contains_key(*pos))
        .map(|(pos, voxel)| {
            let color = Color::srgb(voxel.color[0], voxel.color[1], voxel.color[2]);
            (*pos, color)
        })
        .collect();

    // Phase 4: spawn new voxels, recording their entities
    let cube_mesh = world_res.cube_mesh.clone();
    for (pos, color) in to_add {
        let material = materials.add(color);
        let entity = commands
            .spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                VoxelCube { position: pos },
                Transform::from_xyz(pos[0] as f32, pos[1] as f32 + 0.5, pos[2] as f32),
            ))
            .id();
        world_res.spawned.insert(pos, entity);
    }
}

// --------------------------------------------------------------------------
// Click raycasting: place / erase voxels on the ground
// --------------------------------------------------------------------------

/// Handles left-click (place) and right-click (erase) on the ground plane
/// via a ray from the camera through the cursor.
pub fn handle_click(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    input: Res<ButtonInput<MouseButton>>,
    mut world_res: ResMut<VoxelWorldResource>,
) {
    let (camera, camera_tf) = camera.into_inner();
    let window = *window;

    let just_left = input.just_pressed(MouseButton::Left);
    let just_right = input.just_pressed(MouseButton::Right);
    if !just_left && !just_right {
        return;
    }

    if let Some(cursor) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_tf, cursor) {
            if let Some(point) = ray.plane_intersection_point(
                Vec3::new(0.0, 0.5, 0.0),
                InfinitePlane3d::new(Vec3::Y),
            ) {
                let x = point.x.round() as i32;
                let z = point.z.round() as i32;
                let pos = [x, 0, z];
                let brush_color = world_res.brush_color;

                if just_left {
                    world_res.world.place(pos, brush_color);
                } else {
                    world_res.world.erase(pos);
                }
            }
        }
    }
}

// --------------------------------------------------------------------------
// WASD / QE camera movement
// --------------------------------------------------------------------------

/// Simple WASD / Q E / R F camera movement on the ground plane.
/// Bevy's built-in `TransformSystem` propagates Transform → GlobalTransform.
pub fn move_camera(
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<CameraFollow>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = match cameras.single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };
    let sensitivity = 10.0 * time.delta_secs();
    let mut delta = Vec3::ZERO;

    if input.pressed(KeyCode::KeyW) { delta += Vec3::Z; }
    if input.pressed(KeyCode::KeyS) { delta -= Vec3::Z; }
    if input.pressed(KeyCode::KeyA) { delta -= Vec3::X; }
    if input.pressed(KeyCode::KeyD) { delta += Vec3::X; }
    if input.pressed(KeyCode::KeyQ) { delta -= Vec3::Y; }
    if input.pressed(KeyCode::KeyE) { delta += Vec3::Y; }
    if input.pressed(KeyCode::KeyF) { delta -= Vec3::Y * 2.0; }
    if input.pressed(KeyCode::KeyR) { delta += Vec3::Y * 2.0; }

    if delta.length_squared() > 0.0 {
        let mut pos = transform.translation + delta * sensitivity;
        pos.y = pos.y.max(0.5);
        transform.translation = pos;
    }
}

// --------------------------------------------------------------------------
// Setup helpers
// --------------------------------------------------------------------------

/// Spawns the ground plane, a directional light, and the shared cuboid mesh.
pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_res: ResMut<VoxelWorldResource>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Ground,
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(5.0, 10.0, 5.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Shared 1×1×1 cuboid mesh — reused for every voxel
    world_res.cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
}

/// Spawns the 3D camera looking down at the origin.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 10.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_world_resource_default() {
        let res = VoxelWorldResource::default();
        assert_eq!(res.world.voxel_count(), 0);
        assert_eq!(res.spawned.len(), 0);
        assert_eq!(res.brush_color, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_voxel_world_resource_place() {
        let mut res = VoxelWorldResource::default();
        res.world.place([3, 0, 5], [1.0, 0.5, 0.0]);
        assert_eq!(res.world.voxel_count(), 1);
        assert!(res.world.get([3, 0, 5]).is_some());
        assert_eq!(res.world.get([99, 0, 99]), None);
    }
}
