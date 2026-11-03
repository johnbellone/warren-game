//! Rendering layer: VoxelWorldResource and per-chunk mesh sync.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};
use std::collections::HashMap;

use crate::character::CameraFollow;
use crate::mesh::{collect_chunk_faces, FaceData};
use crate::world::{ChunkPos, VoxelWorld};

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
    /// Per chunk: the mesh entities currently representing it (one per color
    /// group). Rebuilt only for dirty chunks.
    pub spawned_chunks: HashMap<ChunkPos, Vec<Entity>>,
    /// The shared 1×1×1 cuboid mesh handle (created once in setup_environment).
    pub cube_mesh: Handle<Mesh>,
    /// The currently selected brush colour (RGB 0.0–1.0).
    pub brush_color: [f32; 3],
}

impl Default for VoxelWorldResource {
    fn default() -> Self {
        Self {
            world: VoxelWorld::default(),
            spawned_chunks: HashMap::new(),
            cube_mesh: Handle::default(),
            brush_color: [1.0, 0.0, 0.0],
        }
    }
}

// --------------------------------------------------------------------------
// Sync system: per-chunk merged mesh culling
// --------------------------------------------------------------------------

/// Build and sync chunked voxel meshes.
///
/// Replaces the old per-voxel `Mesh3d(Cuboid)` path. For each chunk touched
/// since the last frame, the previous chunk meshes are despawned and a new set
/// of merged meshes is built: one mesh per distinct color, with internal faces
/// culled (across chunk seams via `world.get`). A dirty chunk that became empty
/// is dropped. This turns ~9,700 draw calls into a few dozen, and only the
/// chunks actually edited are re-meshed.
pub fn sync_chunk_meshes(
    mut world_res: ResMut<VoxelWorldResource>,
    mut commands: Commands,
    mut assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dirty = world_res.world.take_dirty();
    if dirty.is_empty() {
        return;
    }

    for cp in &dirty {
        // Despawn this chunk's previous meshes.
        if let Some(old) = world_res.spawned_chunks.remove(cp) {
            for e in old {
                commands.entity(e).despawn();
            }
        }

        // A chunk that became empty has nothing to render.
        if !world_res.world.has_chunk(cp) {
            continue;
        }

        let faces = collect_chunk_faces(&world_res.world, cp);
        let mut entities: Vec<Entity> = Vec::with_capacity(faces.len());
        for fd in &faces {
            let handle = build_chunk_mesh(fd, &mut assets);
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(fd.color[0], fd.color[1], fd.color[2]),
                perceptual_roughness: 0.8,
                metallic: 0.05,
                ..default()
            });
            let entity = commands
                .spawn((
                    Mesh3d(handle),
                    MeshMaterial3d(material),
                    Transform::default(),
                ))
                .id();
            entities.push(entity);
        }
        if !entities.is_empty() {
            world_res.spawned_chunks.insert(*cp, entities);
        }
    }
}

/// Build a `Mesh3d` from a chunk face group, inserting vertex position, normal,
/// UV, and triangle-index attributes.
fn build_chunk_mesh(fd: &FaceData, assets: &mut Assets<Mesh>) -> Handle<Mesh> {
    let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_indices(Indices::U32(fd.indices.clone()))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, fd.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, fd.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, fd.uvs.clone());
    assets.add(mesh)
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

    if input.pressed(KeyCode::KeyW) {
        delta += Vec3::Z;
    }
    if input.pressed(KeyCode::KeyS) {
        delta -= Vec3::Z;
    }
    if input.pressed(KeyCode::KeyA) {
        delta -= Vec3::X;
    }
    if input.pressed(KeyCode::KeyD) {
        delta += Vec3::X;
    }
    if input.pressed(KeyCode::KeyQ) {
        delta -= Vec3::Y;
    }
    if input.pressed(KeyCode::KeyE) {
        delta += Vec3::Y;
    }
    if input.pressed(KeyCode::KeyF) {
        delta -= Vec3::Y * 2.0;
    }
    if input.pressed(KeyCode::KeyR) {
        delta += Vec3::Y * 2.0;
    }

    if delta.length_squared() > 0.0 {
        let mut pos = transform.translation + delta * sensitivity;
        pos.y = pos.y.max(0.5);
        transform.translation = pos;
    }
}

// --------------------------------------------------------------------------
// Setup helpers
// --------------------------------------------------------------------------

/// Builds a grid texture procedurally, returned as a top-down `Image` (RGBA8).
///
/// * background               – dark slate
/// * minor lines (every 1 m)  – dim grey, 1 texel thick
/// * major lines (every 10 m) – bright blue, 2 texel thick
///
/// A single opaque textured plane avoids z-fighting (one depth value) and
/// thin-line flicker (lines are baked into the texture and are multi-pixel).
fn build_grid_texture(
    texels_per_cell: u32,
    cells: u32,
    minor_line: u32,
    major_line: u32,
) -> Image {
    let size = texels_per_cell * cells;
    let mut pixels: Vec<u8> = vec![0u8; (size * size * 4) as usize];

    let bg: [u8; 3] = [16, 16, 24];
    let minor: [u8; 3] = [60, 64, 80];
    let major: [u8; 3] = [100, 165, 255];

    // A major line spans `major_line` texels at every 10-th cell boundary.
    let major_stride = texels_per_cell * 10;

    for y in 0..size {
        for x in 0..size {
            let rgb = if x.rem_euclid(major_stride) < major_line
                || y.rem_euclid(major_stride) < major_line
            {
                major
            } else if x.rem_euclid(texels_per_cell) < minor_line
                || y.rem_euclid(texels_per_cell) < minor_line
            {
                minor
            } else {
                bg
            };

            let i = ((y * size + x) * 4) as usize;
            pixels[i] = rgb[0];
            pixels[i + 1] = rgb[1];
            pixels[i + 2] = rgb[2];
            pixels[i + 3] = 255;
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Spawns a dark ground plane with a tiled wireframe grid overlay, a directional
/// light, and the shared 1×1×1 cuboid mesh.
pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    mut world_res: ResMut<VoxelWorldResource>,
) {
    // Black opaque ground plane, slightly below the grid.
    commands.spawn((
        Mesh3d(meshes.add(
            Plane3d::default().mesh().size(256.0, 256.0)
        )),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.02, 0.06),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.05, 0.0)
            .looking_at(Vec3::ZERO, Vec3::Y)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Ground,
    ));

    // Grid: 1 texel-per-meter, 256 cells → 256 m wide. Lines baked into the
    // texture. minor_line=2, major_line=4 texels keeps lines crisp at distance.
    let grid_image = build_grid_texture(1, 256, 2, 4);
    let grid_texture = textures.add(grid_image);
    let grid_material = materials.add(StandardMaterial {
        base_color_texture: Some(grid_texture),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(
            Plane3d::default().mesh().size(256.0, 256.0)
        )),
        MeshMaterial3d(grid_material),
        Transform::from_xyz(0.0, 0.02, 0.0)
            .looking_at(Vec3::ZERO, Vec3::Y)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // Directional light (shadows off for performance).
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(5.0, 10.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Shared 1×1×1 cuboid mesh handle (retained for any future per-voxel use).
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
        assert_eq!(res.spawned_chunks.len(), 0);
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
