---
type: Architecture
title: Texture Loader Crate Analysis — minecraft_assets vs. Custom vs. Bevy Built-in
description: "Comparison of minecraft_assets crate, Bevy 0.20 built-in texture system, and a custom serde_json parser for loading Minecraft block models and textures into warren-game."
tags: [analysis, crates, minecraft_assets, bevy, texture, atlas, serde_json, block-model]
generated: { by: agent/mcp, at: "2026-11-01T20:00:00Z" }
status: stable
---

## Overview
Evaluate three approaches to parsing Minecraft resource pack formats (blockstates, models, textures) for warren-game's Bevy 0.20 texture loader.

## Option 1: minecraft_assets Crate
**Repo**: BGR360/minecraft-assets-rs, docs.rs/minecraft-assets, Apache-2.0/MIT dual-license

### API (from docs.rs)
```rust
use minecraft_assets::api::{AssetPack, ResourceProvider, FileSystemResourceProvider};
use minecraft_assets::schemas::{BlockState, Model, Texture, Animation};

// Point at a resource pack folder (Minecraft folder structure)
let assets = AssetPack::at_path("~/.minecraft/assets/");
let states = assets.load_blockstates("oak_planks")?;
let variants = states.variants()?[String::new()];
let model = variants.models()[0];  // Model struct
```

### Features
- Parses: blockstates/*.json, models/block/*.json, models/item/*.json
- Parses: font/*.json, lang/*.json, particles/*.json
- Parses: shaders/{post,program}/*.json
- Parses: textures/*.mcmeta (animation metadata)
- Parses: sounds.json
- Parses: pack.mcmeta (validation with pack_format)
- **Does NOT render** — only parses and provides data structures
- Works with any Minecraft version (version-agnostic via ResourceProvider)
- Project uses: Brine (Bevy Minecraft client) — proves Bevy integration works

### Pros
- Mature, version-agnostic
- Already parses the exact format needed
- Brine uses it with Bevy — proven
- Well-documented, `#![warn(missing_docs)]`

### Cons
- Adds a dependency (small: ~30KB crate metadata, ~15KB code)
- `pack.mcmeta` requires Minecraft's resource pack format — warren-game may have its own folder structure
- May need adaptation for non-Minecraft vanilla resource packs

### Recommended For
Loading Minecraft vanilla resource packs or resource packs exported in vanilla format.

## Option 2: Bevy 0.20 Built-in Texture System
**No external crate** — use Bevy's own types.

### Key Types (from docs.rs/bevy/0.20.1)
```rust
use bevy::prelude::*;
use bevy::render::texture::DefaultImageSampler;

// Load PNG → Image
let handle: Handle<Image> = asset_server.load("textures/block/stone.png");

// Create a texture atlas from a single image
let atlas_layout = TextureAtlasLayout::grid(
    Vec2::new(64.0, 64.0),  // total image size
    TextureAtlasLayoutOptions {
        spacing: UVec2::ZERO,
        padding: UVec2::ZERO,
        ..default()
    }
);
// Or: TextureAtlasLayout::from_grid(cols, rows, image_size, spacing)

// Create a texture atlas from individual images
let atlas_layouts = TextureAtlasLayout::from_squares(
    vec![(Vec2::new(16.0, 16.0), 0); num_textures],
    // ...
);

// Create a MeshMaterial3d with atlas
let material = StandardMaterial {
    base_color: Color::WHITE,
    base_texture: Some(atlas_handle),
    texture_atlas_layout: Some(atlas_layout),
    unlit: false,
    ..default()
};

// Apply to a Cuboid mesh
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),  // 1x1x1
    MeshMaterial3d(material),
    Transform::from_xyz(0.0, 0.5, 0.0),
));
```

### Features
- `Handle<Image>` — Bevy assets, PNG decode built-in (`bevy::asset::AssetServer<Image>`)
- `TextureAtlasLayout` — UV region computation from grid or from individual squares
- `Image` — wraps RGBA8/RGBA16 image data, uploaded to GPU via `bevy_render`
- `StandardMaterial::base_texture` + `texture_atlas_layout` — applies atlas UVs to all faces uniformly
- **Limitation**: `StandardMaterial` uses one atlas for all 6 faces. To get different textures per face (like Minecraft's 6 face textures), you need a custom `PbrMaterial`/shader or use `Mesh3d` with per-face vertex UVs

### Pros
- Zero new dependencies
- GPU upload handled by Bevy's async asset system
- Proven and documented
- Fast: atlas layout computed once, stored in `Handle<TextureAtlasLayout>`

### Cons
- **Key limitation**: `StandardMaterial` applies the same texture to all 6 faces uniformly. For different textures per face (Minecraft's cube_all vs. cube model), need a custom `PbrMaterial` with a shader that selects the face texture by face direction
- Or: use `Mesh3d` vertex UVs — manually set vertex UV coordinates for each face of the cube from model UV data
- `TextureAtlasLayout::from_squares` creates a single atlas image from a grid of individual images

### Recommended For
MVP with simple per-voxel textures (same texture on all faces, or 6-image atlas with `Mesh3d` vertex UVs).

## Option 3: Custom serde_json Parser
Build a minimal parser for the 3 file formats:
1. `assets/models/block/<name>.json` — Java block model
2. `assets/blockstates/<name>.json` — block states JSON
3. `assets/textures/*.mcmeta` — animation metadata

### Minimal Struct
```rust
use serde::Deserialize;
use bevy::render::texture::TextureAtlasLayout;

#[derive(Debug, Deserialize, Clone)]
pub struct BlockModel {
    pub parent: Option<String>,   // "block/cube_all", "block/cube", "./other_model.json"
    #[serde(default)]
    pub textures: std::collections::HashMap<String, String>,  // "side" => "block/stone_side", "all" => "block/stone"
    #[serde(default)]
    pub elements: Vec<ModelElement>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    #[serde(default)]
    pub rotation: Option<Rotation>,
    pub faces: std::collections::HashMap<String, Face>,  // "north", "south", "east", "west", "up", "down"
}

#[derive(Debug, Deserialize, Clone)]
pub struct Face {
    pub texture: Option<String>,  // "#side", "#all", "block/stone_top", etc.
    #[serde(default)]
    pub uv: Option<[f32; 4]>,     // [u0, v0, u1, v1] in 0-16 pixel space
    #[serde(default = "default_uv")]
    fn default_uv() -> Option<[f32; 4]> { None }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rotation { pub angle: f32, pub axis: String, pub origin: [f32; 3] }
```

### Pros
- Zero dependencies added
- Only supports 2-3 JSON formats needed for MVP
- Easier to extend
- No learning curve of a third-party crate

### Cons
- May miss Minecraft format edge cases
- Blockstate parsing (which variants → which models) is non-trivial
- No animation metadata parsing (`.mcmeta`)

### Recommended For
MVP with only `cube_all`/`cube`/`cube_column` cube parents. No blockstates, no animation.

## Comparison Matrix
| Feature | minecraft_assets | Bevy built-in | Custom serde_json |
|---|---|---|---|
| Block model JSON parsing | ✅ Full | ❌ Not available | ✅ (cube_all/cube only) |
| Blockstates JSON | ✅ Full | ❌ No | ❌ (skip for MVP) |
| Texture atlas layout | ❌ (parse only) | ✅ `TextureAtlasLayout` | ✅ via Bevy |
| .mcmeta animation | ✅ | ❌ (manual) | ❌ (skip for MVP) |
| pack.mcmeta validation | ✅ | ❌ | ❌ |
| New dependencies | 1 | 0 | 0 |
| Line count added to project | 0 (use as dependency) | 0 | ~150 lines |
| Bevy integration | Via Brine (proven) | Native | Manual |
| MVP suitability | Overkill | ✅ Best | ✅ Best (simplest) |
| Long-term scalability | ✅ | ✅ | ❌ |

## Recommendation for warren-game MVP
**Use Bevy built-in types** (`Handle<Image>`, `TextureAtlasLayout`, `StandardMaterial`) with a **custom 150-line serde_json parser** for the block model JSON only.

Rationale:
- `minecraft_assets` requires Minecraft's exact resource pack structure and pack.mcmeta validation — an overkill for an MVP
- Bevy's `TextureAtlasLayout` and `Handle<Image>` do exactly what's needed for the atlas
- A minimal `serde_json` parser for `elements[].faces[].texture` covers cube_all/cube/cube_column
- Zero new dependencies beyond the `serde`/`serde_json` already in the workspace

## Future Migration
- When blockstates are needed: add `minecraft_assets` dependency (version-agnostic, proven with Brine/Bevy)
- When animations needed: add `.mcmeta` parsing to the custom parser
- When CTM needed: add `ctm` parsing, generate variant atlas at load time
