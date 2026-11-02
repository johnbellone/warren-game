# Texture Loader Architecture Plan

**Status**: Plan only — no implementation code yet. This document is the input to a future goal.
**Date**: 2026-11-01
**References**: weft-analysis.md, pumpkin-analysis.md, texture-loader-crates.md, texture-loader-architecture.md

---

## 1. Objective
Add Minecraft-style textured cubes to the warren-game Bevy 0.20 voxel renderer:
- Load 16×16 PNG textures into a shared GPU texture atlas
- Parse Java Edition block model JSON (`elements`, `faces`, `textures`, `parent`)
- Render textured cubes instead of solid-colour cubes, without changing the protocol contract
- Support `cube_all`, `cube`, `cube_column` model parents
- Mantine UI: texture picker replaces the colour-only picker

---

## 2. Current State

The warren-game crate renders voxels as solid-colour `StandardMaterial` cubes:
```rust
// game/src/rendering.rs — current
StandardMaterial::from(Color::srgb(voxel.color[0], voxel.color[1], voxel.color[2]))
```

`Voxel { position: [i32;3], color: [f32;3], asset_type: AssetType }`
`VoxelEdit { peer_id, position, color, erase }`

No textures, no atlas, no model parsing.

---

## 3. Target State (MVP)

- `Voxel { position, color, texture_id: Option<String>, asset_type }`
- `VoxelEdit { peer_id, position, color, texture_id: Option<String>, erase }`
- `TextureAtlasManager` — Bevy resource holding `Handle<TextureAtlasLayout>` + `Handle<Image>` + `HashMap<String, UvRegion>`
- `BlockRegistry` — Bevy resource mapping block name → block model JSON → face definitions
- Textured cube: `Mesh3d(Cuboid)` with per-vertex atlas UVs, `MeshMaterial3d` with `base_texture` + `texture_atlas_layout`
- Mantine `TexturePicker` component → Tauri `set_texture` command → engine process state

---

## 4. Crate Choice

**Decision: Bevy built-in + minimal `serde_json` parser for MVP.** No new dependencies.

| Option | Verdict | Reason |
|---|---|---|
| `minecraft_assets` crate | Defer | Proven with Brine/Bevy, but requires vanilla pack structure; add when blockstates are needed |
| Bevy built-in | ✅ MVP | `Handle<Image>`, `TextureAtlasLayout`, `StandardMaterial::base_texture` cover everything |
| Custom `serde_json` parser | ✅ MVP | ~150 lines for `BlockModel`/`ModelElement`/`ModelFace` structs; covers cube_all/cube/cube_column |

When blockstates (multiple variants per block) are needed: add `minecraft_assets` (version-agnostic, Apache-2.0/MIT).

---

## 5. Module Structure

```
game/src/
  texture/
    mod.rs
    atlas.rs       — TextureAtlasManager, AtlasRegistry, build_atlas()
    model.rs       — BlockModel, ModelElement, ModelFace, parse_cube_parent()
    registry.rs    — BlockRegistry: block name → model path → atlas texture
  world.rs         — Voxel: add texture_id: Option<String>
  rendering.rs     — apply_voxel_to_entity: check texture_id first, fall back to colour
```

---

## 6. Bevy System Layout

```rust
// Startup
load_textures     // Build atlas from assets/textures/block/ folder
load_block_models // Parse assets/models/block/*.json, resolve parent chains

// Update
update_voxel_materials   // Apply texture atlas texture to existing voxels when texture_id set
apply_voxel_to_entity    // Existing system, extended: use atlas texture when texture_id is Some
```

---

## 7. Key APIs

### Bevy 0.20 — Texture Atlas
```rust
use bevy::render::texture::{TextureAtlasLayout, TextureAtlasLayoutOptions};

// From a grid of individual square textures:
let layout = TextureAtlasLayout::from_squares(
    vec![(Vec2::new(16.0, 16.0), 0); num_textures],
    TextureAtlasLayoutOptions { spacing: UVec2::splat(0), padding: UVec2::splat(0), ..default() }
);
let image_handle = asset_server.load("atlas.png");

// Per-voxel material:
let mat = StandardMaterial {
    base_color: Color::WHITE,
    base_texture: Some(image_handle),
    texture_atlas_layout: Some(layout_handle),
    unlit: false,
    ..default()
};
```

### Custom serde_json parser
```rust
#[derive(Deserialize)]
pub struct BlockModel {
    pub parent: Option<String>,
    pub textures: HashMap<String, String>,
    #[serde(default)]
    pub elements: Vec<ModelElement>,
}
```
~150 lines, covers `cube_all` / `cube` / `cube_column` parents.

---

## 8. Migration Steps (Ordered)

| Step | Task | Deliverable |
|---|---|---|
| 1 | Add `texture_id: Option<String>` to `Voxel` + `VoxelEdit` in `protocol/` | `cargo test -p warren-protocol` |
| 2 | Create `game/src/texture/` with `atlas.rs`, `model.rs`, `registry.rs` | Module compiles |
| 3 | Implement `load_textures` — build atlas from PNG folder | Atlas resource populated |
| 4 | Implement `load_block_models` — parse JSON, resolve parents | BlockRegistry resource populated |
| 5 | Extend `apply_voxel_to_entity` — use atlas when `texture_id.is_some()` | Textured cube renders |
| 6 | Add `set_texture` Tauri command + `TexturePicker` Mantine component | UI shows texture picker |
| 7 | Two-client test — Client A places textured voxel, Client B sees it | Live test passes |

---

## 9. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| `StandardMaterial` can't do per-face textures natively | High | Use per-vertex UVs on a custom `Mesh` (Option B), test with 1 texture first |
| UV Y-axis flipping (atlas top-down vs. Bevy origin) | Medium | Test empirically; flip v-coords if blocks appear upside down |
| Texture atlas border bleed (neighbouring textures bleeding into each other) | High | Use `spacing: 1` in `TextureAtlasLayout` to add border pixels |
| 6 faces × N voxels = 6× mesh count in Option A | High | Go to per-vertex UV `Mesh3d` (Option B) at scale, not per-face quads |
| `VoxelEdit.texture_id` null vs. `None` in JSON | Low | serde handles `Option<T>` as null in JSON; test round-trip |

---

## 10. Mantine UI — TexturePicker

```
┌─ Texture ──────────────┐
│ [dropdown ▼]            │
│  ┌──────────────────┐  │
│  │ [thumb] stone    │  │
│  │ [thumb] dirt     │  │
│  │ [thumb] grass    │  │
│  └──────────────────┘  │
│  Use colour instead     │ ← checkbox
│  [colour picker]        │ ← shown when "use colour" checked
└─────────────────────────┘
```

- Texture names from `BlockRegistry`
- On select: `invoke('set_texture', { texture_id })`
- Thumbnails: could be tiny base64 PNGs or just text names for MVP

---

## 11. References

- Weft engine: `knowledge/weft-analysis.md`
- Pumpkin server: `knowledge/pumpkin-analysis.md`
- Crate options: `knowledge/texture-loader-crates.md`
- Full architecture: `knowledge/texture-loader-architecture.md`
- Weft source: https://github.com/Weft-Rust/Weft/tree/master (crates/render/src/lib.rs)
- Bevy docs: https://docs.rs/bevy/0.20.1/bevy/render/texture/
- minecraft_assets: https://docs.rs/minecraft-assets/
