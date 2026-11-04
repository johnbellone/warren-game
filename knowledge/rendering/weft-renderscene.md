---
type: Architecture
title: "Warren-Game Bevy 0.20 Rendering & Animation Decisions"
description: "Verified Bevy 0.20 rendering, physics (avian3d 0.8), 10-deep procedural voxel ground, 3x resolution, and procedural skybox decisions for warren-game."
tags: [bevy, grid, texture, animation, character, decisions, cargo]
generated: { by: agent/cli, at: "2026-11-03T20:00:00Z" }
status: stable
---

## Summary
warren-game (Bevy 0.20) grid rendering and character walk-animation designs, verified by compiling and running the `warren-game-app` binary on macOS/Metal.

## Grid Blink Fix
- Old grid = ~400 overlapping thin `Cuboid` bar entities per direction → caused both depth/z-fighting with the ground and sub-pixel thin-line flicker (many overlapping quads with depth).
- Fix: a SINGLE opaque textured plane on top of a single dark ground plane.
  * Ground: `Plane3d` 256m at `y = -0.05`, `StandardMaterial { base_color: near-black, unlit: true }`.
  * Grid: `Plane3d` 256m at `y = 0.02` (7 cm above ground), `base_color_texture: Some(grid_image)`, `unlit: true`.
  * `build_grid_texture(texels_per_cell=1, cells=256, minor_line=2, major_line=4)` builds a 256×256 `Image` (RGBA8 sRGB): bg dark, minor lines every 1 m, major lines every 10 m, baked into pixels. 1 texel = 1 m, lines are multi-pixel → no flicker even when zoomed.
- Two-plane separation + single textured plane is the definitive fix; unlit materials avoid lighting modulating the color.

## Bevy 0.20 API Corrections (verified by compile)
- `StandardMaterial` fields: `base_color: Color`, `base_color_texture: Option<Handle<Image>>` (NOT `color` or `base_texture`), `emissive: LinearRgba`, `unlit: bool`, `alpha_mode: AlphaMode`. Convert sRGB→linear for `emissive` via `Color::srgb(...).into()` or `Srgba`/`to_linear()`.
- No `bevy::color::paletted` module, no `.to_linear_rgba()` on `Color` (method is `to_linear()`).
- Texture construction: `bevy::asset::RenderAssetUsages`, `bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat}`, `Image` is in the prelude from `bevy::bevy_image`.
- `Extent3d { width, height, depth_or_array_layers: 1 }` — the field is `depth_or_array_layers`, NOT `depth_or_array_size`.
- `Image::new(Extent3d, TextureDimension::D2, Vec<u8> rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())`.
- No `UnlitMaterial` type in 0.20; use `StandardMaterial { unlit: true, ..default() }`.
- `Vec3::normalized()` needs a Normalized import in 0.20; use `v / v.length()` instead.
- `Single<(&Camera,&GlobalTransform)>` is fine when only one camera exists; `Query::single()` replaced `get_single()`.

## Asset Path (verified)
- `AssetPlugin.file_path` in Bevy 0.20 resolves relative to the BINARY directory (`target/debug/`), not CWD or `CARGO_MANIFEST_DIR`.
- Fix in `game/src/main.rs`: set `AssetPlugin { file_path: <absolute path to game/assets>, ..default() }` using `std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")`.
- Without this, glTF model load reports `Path not found: .../target/debug/game/assets/...` and the model never renders.

## Character Walk Animation Design (verified)
- `charcter.rs`: `CharacterState { moving, manual, last_auto, position, spinning }`.
- `move_character` sets `state.moving` from horizontal (WASD) input and rotates the character to face the movement direction (yaw = `dir.x.atan2(dir.z)`; +Z assumed forward — flip sign if model faces -Z).
- New `auto_animation` system: plays Walk (node 1) while moving, Idle (node 0) otherwise; uses `last_auto` index guard so it only replays on transition, not every frame.
- `1`–`7` keys call `switch_animation` which sets `state.manual = true` (locks a clip); press `M` to return to auto walk/idle.
- The glTF root entity (`WorldAssetRoot` + `Character`) must ALSO have an explicit `Transform::default()` for `Query<&mut Transform, With<Character>>` to match; the glTF scene children get their own Transforms but the root doesn't get one automatically.
- `Query<Option<&T>>` matches ALL entities (returns None for those without T), so `.is_empty()` is not a "does a T exist" test. Use `Query<&T>` for existence checks.

## Per-chunk Face-culling Mesh (Phase 2, verified by compile + tests)
- Replaced per-voxel `Mesh3d(Cuboid)` (~9,699 draw calls for the stylized Capitol) with one
  merged mesh per **chunk × distinct color** (`game/src/mesh.rs::collect_chunk_faces`). Result: ~16
  draw calls, 3 orders of magnitude fewer. Only chunks that actually change are re-meshed.
- A face is emitted **only when the neighbouring voxel is empty**, looked up via `VoxelWorld::get`
  across chunk seams. This culls internal faces AND seam faces correctly (unit-tested with voxels at
  [31,0,0]/[32,0,0]).

### Bevy 0.20 mesh construction (verified by compile)
- `bevy_render::render_resource::PrimitiveTopology` (NOT `Primitive` / `Primitive2d`); the tri-list
  value is `PrimitiveTopology::TriangleList`.
- Build a raw mesh with the builder form (not `insert_*`, which also work pre-extraction but the
  canonical idiom is the chained builder):
  ```rust
  let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
      .with_inserted_indices(bevy::mesh::Indices::U32(indices))
      .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions) // Vec<[f32;3]>
      .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)     // Vec<[f32;3]>
      .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)          // Vec<[f32;2]>
      .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);     // Vec<[f32;4]> Float32x4
  ```
  `Mesh::setAttribute`/`with_inserted_attribute` panic if the Rust slice type does not match the
  attribute's declared `VertexFormat` — POSITION/NORMAL are `Float32x3` (use `Vec<[f32;3]>`),
  UV_0 is `Float32x2` (`Vec<[f32;2]>`), COLOR/TANGENT are `Float32x4`.
- `Mesh::new(primitive_topology, asset_usage)` is the 2-arg form; `Mesh::new()` (no args) also exists.
- `Indices` is an enum: `Indices::U32(Vec<u32>)` / `Indices::U16(Vec<u16>)` from `bevy_mesh` via
  `bevy::mesh::Indices`, imported explicitly (`use bevy::mesh::Indices;`).

### Per-vertex color in Bevy 0.20 (verified)
- `Mesh::ATTRIBUTE_COLOR` (`VertexFormat::Float32x4`) EXISTS, but there is NO `StandardMaterial`
  per-material vertex-color toggle and NO `VertexColors` component / `VertexColors` field in
  `bevy_pbr-0.20.1` / `bevy_render-0.20.1`. `StandardMaterial` has no `per_vertex_color` field.
- Decision (verified): rather than fight the vertex-color render path, use **one mesh per distinct
  color** (a single-color `StandardMaterial { base_color: Color::srgb(r,g,b), ..default() }` per
  color group). The generated palette (stone/copper/glass/gold, ~4 colors) yields ~4 meshes/chunk.
- Float colors cannot be `HashMap` keys (`f32: !Hash/Eq`); bucket colors by a quantized `[u8;3]`
  key (`(c*255+0.5).round() as u8` per channel) — lossless for the fixed palette.

### Dirty-chunk tracking (verified pattern)
- `VoxelWorld` gained `dirty: HashSet<ChunkPos>`, `mark_dirty(pos)` (dirties the voxel's own chunk
  + up to 6 face-neighbour chunks, since a seam face belongs to both), `take_dirty()`, and
  `has_chunk`/`get_chunk`. `place`/`erase` call `mark_dirty`. `sync_chunk_meshes` drains
  `take_dirty()`, despawns each dirty chunk's old mesh entities, and rebuilds from
  `collect_chunk_faces`, recording entities per chunk in `spawned_chunks: HashMap<ChunkPos,Vec<Entity>>`.

## Avian3D Physics + Procedural Ground + Sky (Phase 3, verified by compile + 37 tests + runtime smoke)

### avian3d 0.8 integration (verified)
- `avian3d = "0.8"` compiles cleanly against Bevy 0.20 on macOS ARM64; adds transitive Parry3d native deps (no pure-Rust fallback needed).
- `PhysicsPlugins::default()` must be added to `WarrenGamePlugin::build` BEFORE calling `app.init_resource::<PhysicsIslands>()` — the physics island resource isn't initialised by `DefaultPlugins`, so ordering matters.
- **Test app pitfall** (verified by a failing test): `app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()))` is NOT sufficient for `character_falls_under_gravity_and_rests_on_floor` — the `ColliderCachePlugin::update_collider_cache_from_asset_events` system reads `MessageReader<AssetEvent<Mesh>>`, which requires `Mesh` to be registered as an asset type. `bevy::mesh::MeshPlugin` must be added to the test app's plugin set alongside `PhysicsPlugins` + `AssetPlugin` — otherwise `app.update()` panics with "Message not initialized: asset_events". This is not a bug in the main app (which uses `DefaultPlugins` that includes `MeshPlugin`), only an oversight when building a minimal test app.
- `Gravity` resource default is `Vec3::ZERO`; must be explicitly set in `setup_physics` via `*gravity = Gravity(Vec3::new(0.0, -9.8, 0.0))` — the `init` does not override it when a default value is the zero vector.
- `Collider::cuboid(x, y, z)` takes the FULL x/y/z dimensions (not half-extents); internally calls `Cuboid::new(x * 0.5, y * 0.5, z * 0.5)`. The character collider is `Collider::cuboid(0.9, 1.8, 0.9)` = ~1.8 m tall.
- `CharacterState.position` is updated by `sync_character_state` (a system that runs AFTER `PhysicsStepSystems::Finalize`), so `camera_follow_character` reads `state.position` (not a direct `Transform` read), avoiding a `&`/`&mut` static conflict on the dynamic body's own `Transform`.

### 10-deep procedural voxel ground (verified)
- `build_ground(half=GROUNd_HALF, depth=GROUND_DEPTH=10)` in `neighborhood.rs` fills a solid slab from `y=-depth` to `y=-1`, `[-half..=half]` in x/z. No `Plane3d`, no `build_grid_texture`, no glow plane — entirely replaced by voxel cubes meshed per-chunk via `collect_chunk_faces`.
- No z-fighting or sub-pixel flicker possible: the ground is 10 stacked `Cuboid`-derived `TriangleList` meshes, not overlapping thin quads. The `sync_chunk_meshes` dirty-chunk pattern re-draws only changed chunks.
- `GROUND_HALF` (plaza half-extent) was raised to `64` (from `32`) so the static Avian collider slab in `setup_physics` (`Collider::cuboid(GROUND_COLLIDER_HALF*2, GROUND_DEPTH, GROUND_COLLIDER_HALF*2)` at `y = -GROUND_DEPTH/2`) covers the full voxel ground. The top face of the collider sits at `y=0` so the character rests ON the voxel ground's top surface, not through it.

### 3× voxel resolution (verified)
- `SCALE = 3` in `neighborhood.rs`; every voxel coordinate is multiplied by `SCALE`, and heights/radii are `* SCALE / SCALE`-normalised. This shrinks voxel size to ~1/3 m (from 1 m at SCALE=1), giving ~27× more voxels per unit cube — 381,129 voxels in 49 chunks for the Capitol + ground, up from ~9,699 at SCALE=1.
- `resolution_scale_is_three()` unit test asserts `SCALE == 3`.

### Procedural sky sphere (verified)
- `Sky` marker component + `build_sky()` in `rendering.rs`. No `Sphere` primitive exists in `bevy_scene` at 0.20; the sphere geometry is built by hand with `make_sky_sphere_mesh(radius=480.0, segments=48, slices=24)`: a UV sphere with `segments * slices` quads, closed at both poles (triangles fan from pole vertices to the rings). No degenerate quads or missing faces.
- Sky material: `StandardMaterial { base_color: Color::WHITE, unlit: true, base_color_texture: Some(sky_tex), alpha_mode: AlphaMode::Opaque, ..default() }` on a `Mesh` of radius 480 m, placed at `Vec3::ZERO`. The `Mesh` component is `Sphere(480)` or a hand-built mesh — both work. No `Sphere::new()` primitive needed.
- Sky texture: 256×512 `Rgba8UnormSrgb` gradient. Horizon = pale blue `Color::srgb(0.6, 0.8, 1.0)`, zenith = dark blue `Color::srgb(0.05, 0.1, 0.4)`. The gradient is built by lerping RGB per row: `lerp(horizon_color, zenith_color, t)` where `t = row / (height-1)`. Top row = zenith, bottom row = horizon. `sky_gradient_texture_is_bluer_at_zenith` test asserts `rgba(top).2 > rgba(bottom).2` (zenith is bluer = higher blue channel).
- `Sky` marker is used by the camera or any future system that wants to reference the sky entity; `setup_sky` inserts `Sky` and a log line confirms "sky entity spawned at radius 480".
- `bevy::picking::PickingPlugin` is NOT needed for the main app (it uses `DefaultPlugins`, which includes it). It IS needed when building a minimal test app that uses `PhysicsPlugins::default()` — add `bevy::picking::PickingPlugin` and `bevy::mesh::MeshPlugin` explicitly (see the test pitfall above).

### Bevy 0.20 test app gotcha for Avian (verified by compile + test failure + fix)
- A minimal test app using `app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default(), AssetPlugin::default()))` will panic on `app.update()` with "Message not initialized" because `ColliderCachePlugin` reads `MessageReader<AssetEvent<Mesh>>` — which requires `bevy::mesh::MeshPlugin` to register `Mesh` as an asset type. Fix: add `bevy::mesh::MeshPlugin` to the plugin tuple in any minimal Avian test app. `PickingPlugin` is not the cause; it was the `AssetEvent<Mesh>` that was missing.

# Related Concepts
- [Weft Voxel Engine — Texture Atlas & Resource Pack Analysis](../weft-analysis.md): Corrects and extends the weft-analysis Bevy 0.20 translation table with actual verified API (base_color_texture, emissive LinearRgba, single-plane textured grid, auto-walk animation)
- [Game Component (Voxel World)](../components-game.md): Concrete implementation of the playable character entity and environment rendering described by components-game
