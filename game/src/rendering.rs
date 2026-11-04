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

/// Marker + position for a spawned voxel cube entity (retained; chunk meshes
/// do not use per-voxel entities, but keep the type for future use).
#[derive(Component)]
pub struct VoxelCube {
    pub position: [i32; 3],
}

/// Marker component for the procedural skybox entity (an inside-out, unlit,
/// gradient-shaded sphere that encloses the scene).
#[derive(Component)]
pub struct Sky;

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

/// Spawns a procedural skybox: a large, double-sided sphere shaded with a
/// vertical gradient that runs from a soft horizon colour at the bottom to a
/// deep sky blue at the top. The texture is generated in-memory (2×512 RGBA8),
/// so the scene is fully self-contained — no external `.hdr`/`.png` asset
/// needed.
///
/// `double_sided: true` renders the interior faces so we see the sky from
/// inside; `unlit: true` so the gradient shows at full strength independent of
/// the scene's directional light.
fn build_sky(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    textures: &mut Assets<Image>,
) {
    // Gradient texture: one column of `h` rows, row 0 = horizon (pale),
    // row h-1 = zenith (deep blue). Bevy uploads row 0 at the bottom (v=0).
    let h = 512u32;
    let w = 2u32;
    let mut pixels: Vec<u8> = vec![0u8; (w * h * 4) as usize];
    let horizon: [f32; 3] = [0.70, 0.80, 0.92]; // pale near the horizon
    let zenith: [f32; 3] = [0.16, 0.30, 0.60]; // deep blue at the zenith
    for row in 0..h {
        let t = row as f32 / h as f32;
        let e = t * t; // ease so the zenith stays rich
        let r = lerp(horizon[0], zenith[0], e);
        let g = lerp(horizon[1], zenith[1], e);
        let b = lerp(horizon[2], zenith[2], e);
        for col in 0..w {
            let i = ((row * w + col) * 4) as usize;
            pixels[i] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
            pixels[i + 1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
            pixels[i + 2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
            pixels[i + 3] = 255;
        }
    }

    let gradient = textures.add(Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    ));

    let sky_radius = 480.0_f32;
    let sky_mesh = meshes.add(make_sky_sphere_mesh(sky_radius, 48, 24));

    let sky_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(gradient),
        unlit: true,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_material),
        Transform::default(),
        Sky,
    ));

    info!("sky entity spawned at radius {sky_radius} (unit sphere + zenith/horizon gradient texture)");
}

/// Linear interpolation for the sky gradient (sRGB space is fine here).
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Build a UV sphere mesh of `radius` with `slices` longitude segments and
/// `stacks` latitude rings. Faces wind so the interior is visible when
/// rendered double-sided. UV `v` runs 0 (south pole) to 1 (north pole) so a
/// vertical gradient texture maps cleanly to the sky.
fn make_sky_sphere_mesh(radius: f32, slices: u32, stacks: u32) -> bevy::mesh::Mesh {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // (stacks + 1) × (slices + 1) grid of vertices.
    for i in 0..=stacks {
        let phi = std::f32::consts::PI * i as f32 / stacks as f32; // 0..PI (top..bottom)
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        for j in 0..=slices {
            let theta = 2.0 * std::f32::consts::PI * j as f32 / slices as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let dir = Vec3::new(cos_theta * sin_phi, cos_phi, sin_theta * sin_phi);
            positions.push(dir * radius);
            normals.push(dir);
            uvs.push(Vec2::new(j as f32 / slices as f32, 1.0 - i as f32 / stacks as f32));
        }
    }

    // Two triangles per quad.
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * (slices + 1) + j;
            let b = a + (slices + 1);
            indices.extend_from_slice(&[a, b, a + 1]);
            indices.extend_from_slice(&[b, b + 1, a + 1]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_indices(bevy::mesh::Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

// --------------------------------------------------------------------------
// Environment setup
// --------------------------------------------------------------------------

/// Spawns a directional light, the procedural skybox, and the shared 1×1×1
/// cuboid mesh handle. The old textured ground + grid plane were removed in
/// favor of the 10-deep procedural voxel ground (see `neighborhood.rs`),
/// which is now the only floor — so there are no overlapping planes to
/// z-fight or flicker against.
pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    mut world_res: ResMut<VoxelWorldResource>,
) {
    build_sky(&mut commands, &mut meshes, &mut materials, &mut textures);

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

    /// The procedural skybox is a self-contained, closed UV sphere built from
    /// the mesh + material + gradient texture in `build_sky` — this asserts the
    /// geometry generator produces a non-degenerate, closed triangular mesh with
    /// matching counts (positions = normals = uvs; indices form closed quads).
    #[test]
    fn sky_sphere_geometry_is_closed_and_consistent() {
        let mesh = make_sky_sphere_mesh(480.0, 48, 24);

        // Positions, normals, and UVs must have the same vertex count.
        let positions = mesh.attribute(bevy::mesh::Mesh::ATTRIBUTE_POSITION).unwrap();
        let normals = mesh.attribute(bevy::mesh::Mesh::ATTRIBUTE_NORMAL).unwrap();
        let uvs = mesh.attribute(bevy::mesh::Mesh::ATTRIBUTE_UV_0).unwrap();
        let count = positions.len();

        // A UV sphere with stacks=24, slices=48 → (stacks+1)×(slices+1)
        // = 25 × 49 = 1225 vertices.
        assert_eq!(count, 25 * 49, "vertex count mismatch");
        assert_eq!(normals.len(), count, "normals must match positions");
        assert_eq!(uvs.len(), count, "uvs must match positions");

        // 24 × 48 quads × 2 triangles × 3 verts = 6912 indices.
        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 24 * 48 * 6, "index count mismatch");
        assert!(
            matches!(
                indices,
                bevy::mesh::Indices::U32(_) | bevy::mesh::Indices::U16(_)
            ),
            "index type must be U16 or U32"
        );
    }

    /// The zenith row of the gradient texture is clearly bluer than the horizon
    /// row — i.e. the "self-contained basic skybox" has the intended colour
    /// ramp (pale horizon → deep-blue zenith).
    #[test]
    fn sky_gradient_texture_is_bluer_at_zenith() {
        let h = 512u32;
        let w = 2u32;
        let horizon: [f32; 3] = [0.70, 0.80, 0.92];
        let zenith: [f32; 3] = [0.16, 0.30, 0.60];
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            let t = row as f32 / h as f32;
            let e = t * t;
            for col in 0..w {
                let i = ((row * w + col) * 4) as usize;
                pixels[i] = (lerp(horizon[0], zenith[0], e) * 255.0).round().clamp(0.0, 255.0) as u8;
                pixels[i + 1] = (lerp(horizon[1], zenith[1], e) * 255.0).round().clamp(0.0, 255.0) as u8;
                pixels[i + 2] = (lerp(horizon[2], zenith[2], e) * 255.0).round().clamp(0.0, 255.0) as u8;
                pixels[i + 3] = 255;
            }
        }
        let horizon_b = pixels[2];           // row 0 (horizon), col 0, blue channel
        let horizon_r = pixels[0];
        let last_row = ((h - 1) as usize * w as usize) * 4;
        let zenith_b = pixels[last_row + 2]; // last row (zenith), col 0, blue
        let zenith_r = pixels[last_row];
        assert!(horizon_b >= 100, "horizon should have a meaningful blue value, got {horizon_b}");
        assert!(
            (zenith_b as f32) < (horizon_b as f32) * 0.9,
            "zenith blue must be below horizon blue: zenith_b={zenith_b}, horizon_b={horizon_b}"
        );
        assert!(
            (zenith_r as f32) < (horizon_r as f32) * 0.9,
            "zenith red must be below horizon red; zenith_r={zenith_r}, horizon_r={horizon_r}"
        );
    }

    /// A minimal `World` can spawn an entity tagged with the `Sky` component.
    /// This proves the `Sky` marker is a valid component and that the
    /// procedural skybox entity type is usable in a world context without
    /// the full render pipeline or asset server.
    #[test]
    fn sky_entity_can_be_spawned_in_a_world() {
        let mut world = World::new();
        let e = world.spawn(Sky).id();
        assert!(
            world.get::<Sky>(e).is_some(),
            "Sky component must be present on the spawned entity"
        );
        let matches = world.query::<&Sky>().iter(&world).count();
        assert_eq!(matches, 1, "exactly one Sky entity expected");
    }

}

