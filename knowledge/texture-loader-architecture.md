---
type: Architecture
title: Texture Loading Architecture for warren-game (Bevy 0.20)
description: "Proposed module structure, data structures, Bevy system layout, and migration path for adding textured cubes to warren-game's voxel world. Covers TextureAtlasManager, BlockModel, TextureAtl, Bevy systems, and VoxelEdit protocol extension."
tags: [architecture, bevy, texture, atlas, voxel, block-model, migration, warren-game]
generated: { by: agent/mcp, at: "2026-11-01T20:00:00Z" }
status: stable
---

## Current State
- `game/src/world.rs`: `Voxel { position: [i32;3], color: [f32;3], asset_type: AssetType }`
- `game/src/rendering.rs`: `apply_voxel_to_entity` — uses `StandardMaterial::from(Color::srgb(r, g, b))` per voxel
- `game/src/app.rs`: `setup_voxels` — spawns `Mesh3d(Cuboid(1,1,1))` + `MeshMaterial3d(StandardMaterial{...})` per voxel
- No atlas, no model JSON, no textures — pure solid colour per cube

## Target State (MVP)
- Voxels with texture atlas IDs instead of / alongside `color`
- Texture atlas: N 16×16 PNGs → 1 `Handle<Image>` via `TextureAtlasLayout`
- Block models: JSON `elements[]` with 6 faces, each face can have a texture name
- Rendering: `Mesh3d(Cuboid)` with per-face vertex UVs OR separate `Mesh3d` per face

### Architecture Options

**Option A: Solid cube + 6 separate faces (simplest)**
- Spawn 6 `Mesh3d` (quad) children per voxel, each with its own `StandardMaterial{base_texture, texture_atlas_layout}`
- UVs come from the block model's face `uv: [u0, v0, u1, v1]` in atlas-space
- Pro: easy to implement, works with Bevy's built-in `StandardMaterial`
- Con: 6× mesh count per voxel; 6× draw calls; not performant at scale

**Option B: Single cuboid mesh with vertex UVs per face (recommended for MVP)**
- Build a custom `Mesh` for each 1×1×1 cube with per-vertex UVs
- UVs computed from atlas region: `[u0/v_scale, 1 - v1/v_scale, u1/v_scale, 1 - v0/v_scale]`
- One `Mesh3d` per voxel; one `MeshMaterial3d` per block texture type
- Pro: 1 mesh per voxel, 1 draw call per texture type, performant
- Con: Need a custom Cuboid builder with per-face UVs; or use Bevy's `Mesh` API directly

**Option C: Instanced rendering (future)**
- Use `bevy::render::mesh::Mesh3d` with a single mesh + `MeshMaterial3d` per texture
- Voxel data in a data structure indexed by `mesh_index`
- Pro: GPU-efficient, hundreds of thousands of voxels
- Con: Complex, not needed for MVP

## Recommended: Option B for MVP, Option C as follow-up

## Data Structures

### New: `game/src/texture/` module

```rust
// texture/mod.rs
pub mod atlas;
pub mod model;
pub mod registry;

// texture/atlas.rs — TextureAtlasManager
use bevy::prelude::*;
use bevy::render::texture::{TextureAtlasLayout, TextureAtlasLayoutOptions};

/// Maps texture names (e.g. "block/stone") to UV regions in a single atlas image.
#[derive(Clone, Deref, DerefMut)]
pub struct TextureAtlasManager(pub Handle<TextureAtlasLayout>);

/// Registry mapping texture names → their region in the atlas
pub struct AtlasRegistry {
    pub layout: TextureAtlasLayout,
    pub image_handle: Handle<Image>,
    /// texture_name e.g. "block/stone" → (atlas_index, region [x,y,w,h])
    pub textures: HashMap<String, (u32, bevy::math::UVec4)>,
}

/// Load all PNGs from a folder, build a single atlas image
pub fn build_atlas(
    asset_server: &AssetServer,
    texture_dir: &str,  // "assets/textures/block/"
    tex_names: &[String],
) -> AtlasRegistry { ... }
```

### New: `game/src/texture/model.rs`

```rust
// texture/model.rs — Block model JSON parsing
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct BlockModel {
    pub parent: Option<String>,       // "block/cube_all", "block/cube", "block/cube_column", etc.
    pub textures: HashMap<String, String>,  // "side" → "block/stone_side"
    #[serde(default)]
    pub elements: Vec<ModelElement>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelElement {
    pub from: [f32; 3],  // lower corner: [x, y, z] in 0-16 pixels
    pub to:   [f32; 3],  // upper corner
    #[serde(default)]
    pub rotation: Option<Rotation>,
    pub faces: HashMap<FaceDirection, ModelFace>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rotation {
    pub angle: f32,
    pub axis: String,       // "x", "y", or "z"
    #[serde(default)]
    pub origin: [f32; 3],
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelFace {
    pub texture: String,  // "#side", "#all", or direct "block/stone_top"
    #[serde(default)]
    pub uv: Option<[f32; 4]>,  // [u0, v0, u1, v1] in 0-16 atlas space
    #[serde(default = "default_uv")]
    pub uv: [f32; 4],
    fn default_uv() -> [f32; 4] { [0.0, 0.0, 16.0, 16.0] }
}

/// Resolve a texture reference like "#side" against the model's textures map
pub fn resolve_texture(ref: &str, textures: &HashMap<String, String>, atlas: &AtlasRegistry) -> Option<UvRegion> { ... }

/// Parse "block/cube_all" parent and return the 6-face cube definition
pub fn parse_cube_parent(parent: &str, textures: &HashMap<String, String>) -> Vec<ModelElement> { ... }
```

### New: `game/src/texture/registry.rs`

```rust
// texture/registry.rs — Block type registry
use std::collections::HashMap;

/// Maps block type ID → its block model + texture
pub struct BlockRegistry {
    /// "stone" → path to "assets/models/block/stone.json"
    pub model_paths: HashMap<&'static str, &'static str>,
    /// "block/stone" → atlas region
    pub textures: HashMap<&'static str, UvRegion>,
}

impl BlockRegistry {
    pub fn get(&self, name: &str) -> Option<(&BlockModel, &[UvRegion])> { ... }
}
```

### Modified: `game/src/world.rs`

```rust
// Add texture_id to Voxel — optional in MVP, backward-compatible
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Voxel {
    pub position: [i32; 3],
    pub color: [f32; 3],            // retained for backward-compat
    pub asset_type: AssetType,
    /// NEW: texture atlas name, e.g. "block/stone_side"
    pub texture_id: Option<String>,
    /// NEW: rotation state for blockstates (future)
    pub block_state: Option<u8>,
}
```

### Modified: `protocol/src/lib.rs`

```rust
// VoxelEdit needs texture_id field for collaborative editing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoxelEdit {
    pub peer_id: String,
    pub position: [i32; 3],
    pub color: [f32; 3],
    pub texture_id: Option<String>,  // NEW — can be null
    pub erase: bool,
}

// Voxel also needs the field for server authoritative data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Voxel {
    pub position: [i32; 3],
    pub color: [f32; 3],
    pub asset_type: AssetType,
    pub texture_id: Option<String>,  // NEW
}
```

## Bevy System Layout

```rust
// app.rs — new systems to add to the Update schedule
.app.add_systems(Startup, load_textures)        // Build atlas
.app.add_systems(Startup, load_block_models)     // Parse block model JSONs
.app.add_systems(Update, update_voxel_materials) // Apply texture IDs to existing voxels
.app.add_systems(Update, apply_voxel_to_entity)  // Keep current system, but check texture_id
```

### load_textures
- Walk `assets/textures/block/` directory (from a config or fixed path)
- For each PNG: load via `asset_server.load()`
- Build a `TextureAtlasLayout` with all texture regions
- Store `TextureAtlasManager` as a Bevy `Resource`

### load_block_models
- Parse `assets/models/block/*.json` via `serde_json::from_str`
- Resolve parent chains (`cube_all` → 6-face definitions)
- Store `BlockRegistry` as a Bevy `Resource`

### apply_voxel_to_entity (modified)
- When `voxel.texture_id.is_some()`, look up the atlas region from `TextureAtlasManager`
- Build a `Mesh3d` with per-face vertex UVs OR use separate face quads for MVP
- Apply `StandardMaterial { base_texture: Some(atlas_handle), texture_atlas_layout: Some(atlas_layout) }`
- For solid colour, keep current behaviour

## Mantine / Tauri Integration

### New IPC command: `set_texture`
```rust
#[tauri::command]
fn set_texture(app: AppHandle, texture_id: String) -> Result<(), String> {
    // Update EditorState
    // Call engine process via some mechanism (WebSocket to Bevy's net thread, or process IPC)
}
```

### Mantine UI: TexturePicker
Replace/extend `ColorPicker.tsx` with a texture picker:
- Dropdown of texture names (from `BlockRegistry`)
- When selected, call `invoke('set_texture', { textureId })`
- Show a small preview thumbnail (can be a base64 PNG from the texture)

### EditorState (in client/src/lib.rs)
```rust
#[derive(Default, Clone)]
struct EditorState {
    color: [f32; 3],
    texture_id: Option<String>,  // NEW
    tool: String,
    peer_id: String,
}
```

## Migration Steps (ordered)

1. [x] Current: `StandardMaterial::from(color)` per voxel
2. Add `texture_id: Option<String>` to `Voxel` and `VoxelEdit` in protocol crate
3. Create `game/src/texture/` module with `atlas.rs`, `model.rs`, `registry.rs`
4. Add `TextureAtlasManager` and `BlockRegistry` as Bevy resources
5. Implement `load_textures` system (build atlas from folder of PNGs)
6. Implement `load_block_models` system (parse cube_all/cube models)
7. Modify `apply_voxel_to_entity` to use atlas texture when `texture_id.is_some()`
8. Add Tauri `set_texture` command to `client/src/lib.rs`
9. Add `TexturePicker` component to `editor/`
10. Test: launch game with one textured block, verify it shows texture in Bevy window

## Risks and Unknowns

| Risk | Impact | Mitigation |
|---|---|---|
| Bevy `StandardMaterial` can't do per-face textures natively | Medium | Build custom `Mesh` with per-vertex UVs (Option B) or use `Mesh3d` per face (Option A) |
| `TextureAtlasLayout::from_squares` may not produce expected grid | Low | Test with 4-texture atlas in `cargo run --example` first |
| Block model JSON has edge cases (offset UVs, rotations, inflate) | Medium | Start with `cube_all` only; add `cube` then `cube_column` incrementally |
| 6 faces × N voxels = 6× mesh count (Option A) | High | Go to Option B (single mesh per voxel) before scale |
| `VoxelEdit.texture_id` needs server broadcast support | Low | `VoxelEdit` already has `color`; add `texture_id` field, server broadcasts it unchanged |
| Atlas UV flipping (GL vs Bevy Y-axis) | Medium | Test with `cube_all` first; flip v-coordinates if needed |
| No atlas in Bevy's default PBR material — UV wrapping at atlas borders | High | `TextureAtlasLayout` with `padding: UVec2::splat(0)` + `spacing: UVec2::splat(1)` to prevent bleed |
| Bevy PNG decode: `AssetServer<Image>` needs a `.png` extension or `Image::load()` | Low | Ensure texture files use `.png` extension; use `asset_server.load_with_ext()` if needed |

## UV Computation
For a 16×16 texture in a N×N atlas grid:
- `atlas_region = (col * 16.0 + padding, row * 16.0 + padding, 16.0, 16.0)`
- UV in vertex space: `u ∈ [col/cols, (col+1)/cols]`, `v ∈ [row/rows, (row+1)/rows]`
- Bevy uses top-left origin (UV (0,0) is top-left) — Minecraft uses top-left too, so no flip needed

For a model face with `uv: [u0, v0, u1, v1]` in 0-16 pixel space:
- Vertex UV: `[u0/16, v0/16, u1/16, v1/16]` scaled into atlas region
- If UVs in [0,16] range and atlas cell is 16×16: direct mapping, just offset by atlas col/row

## VoxelEdit Protocol Extension
```rust
// protocol/src/lib.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoxelEdit {
    pub peer_id: String,
    pub position: [i32; 3],
    pub color: [f32; 3],
    pub texture_id: Option<String>,  // NEW — null for solid colour, Some("block/stone") for textured
    pub erase: bool,
}
```

### Server-side: no changes
`server/src/server.rs`'s `handle_message` already handles `VoxelEdit` position/color/erase.
`texture_id: Option<String>` is just a new field — the server broadcasts it unchanged.
No conflict resolution changes needed (last-write-wins per position still applies).

## Mantine UI Design
### TexturePicker component (replaces/extends ColorPicker)
- Mantine `Select` dropdown with texture names
- Each option: small thumbnail preview + name
- On change: `invoke('set_texture', { textureId: name })`
- When `textureId` is selected: `ColorPicker` becomes disabled (or shows a checkbox "use texture over colour")
- In `ConnectionPanel`: show connection status per peer's texture selection

## Testing Plan
1. `cargo test -p warren-protocol` — verify `VoxelEdit` with `texture_id` serializes/deserializes
2. Create 1 test texture (16×16 solid green PNG) in `assets/textures/block/test.png`
3. Create 1 test model `assets/models/block/test.json` with `cube_all` parent, texture `all: block/test`
4. `cargo run --example textured_cube` — spawn a single textured cube, verify it renders
5. Launch full game — verify atlas loads, block registry populates
6. Two-client test: Client A places a textured voxel, Client B sees it with texture
