---
type: Architecture
title: Weft Voxel Engine — Texture Atlas & Resource Pack Analysis
description: "Deep analysis of Weft-Rust/Weft rendering pipeline, texture system, CTM, animations, resource pack layout, and Bevy 0.20 translation for each concept. Reference for warren-game voxel texture loader."
tags: [analysis, weft, voxels, textures, resource-packs, atlas, ctm, bevy, migration]
generated: { by: agent/mcp, at: "2026-11-01T20:00:00Z" }
status: stable
---

## Overview
Weft is a Rust voxel sandbox engine (Vulkan backend, ash 0.38, gpu-allocator, winit, naga for WGSL).
Not a Bevy project — a custom backend-neutral rendering layer that maps onto Vulkan.
~10 crates, MPL-2.0 license, reads Minecraft resource packs.

## Rendering Pipeline (crates/render/src/lib.rs)
- Backend-neutral: `Backend` trait, implemented by `backends/vulkan`
- No vertex buffers — all data in storage buffers, textures in a bindless table
- Textures: global bindless array of 65536 entries, indexed by 16-bit `TextureId(u32)`
- Samplers: `nearest_clamp`, `linear_clamp`, `nearest_mipped`, `linear_mipped`
- 4 samplers (never tiling, to prevent face-edge seams)
- Shaders: WGSL with a global `PRELUDE` (binding group 0 = textures + samplers)
- Formats: Rgba8Srgb, Rgba8Unorm, R8Unorm, Rgba16Float, Depth32, Surface
- Each texture is its own GPU resource — NOT an atlas
- Face records store a 16-bit index into the bindless texture table

## Texture System
### Basic
- Block textures: 16×16 PNG in `assets/textures/block/`
- Entity skins: 64×32, 64×64 PNG in `assets/textures/entity/`
- Items: flat PNG or `models/item/` Blockbench JSON
- PNG decoding via `png = "0.18"` crate

### Sidecar Files (alongside each PNG, in same folder)
| File | Purpose |
|------|---------|
| `texture.png` | Base texture |
| `texture.anim` | Animation config: `frametime`, `interpolate`, `frames = 0 1 2 3*5 2 1` |
| `texture_e.png` | Emission mask (Weft's own format): alpha × red = glow. Black/transparent = no glow. |
| `texture_s.png` | LabPBR specular: alpha channel = glow, 255 = no emission |
| `texture_overlay.png` | Overlay tinted by block's biome/state color |
| `texture.ctm` | Connected/tiled texture config |

### Animation
- Animated texture = vertical strip of square frames: 16×16 × N frames = 16×(16×N) image
- Default: one tick per frame (20 fps)
- `.anim` file controls: `frametime` (ticks/frame), `interpolate` (blend), `frames` (order with `*N` repeat)

### Emission
- Emission mask (`_e.png` or `_s.png`) makes pixels glow in dark
- `_e`: alpha × red channel = glow (Weft format)
- `_s`: alpha channel = glow (LabPBR format, 255 = no emission)
- `_e` overrides `_s` when both exist
- Glows the texture itself, not the surrounding light

### Overlays
- One overlay per texture, always tinted by block color, never has `_e`
- Used for grass sides: dirt base stays brown, grass fringe gets biome tint

## CTM (Connected Textures — OptiFine/MCPatcher style)
File: `assets/textures/block/<texture>.ctm`, key=value format, `#` for comments
Blocks separated by `[separator]` lines, first match wins.

### Methods
| Method | Tiles | Use case |
|--------|-------|----------|
| `ctm` (full) | 47 | Full 4-neighbour edge+corner stitching |
| `compact` | 5 | Same result, only 5 tiles (4-quadrant split) |
| `horizontal` | 4 | Connect left-right (logs, bookshelves) |
| `vertical` | 4 | Connect up-down (sugar cane) |
| `h+v` / `v+h` | 8 | Both directions |
| `top` | 1 | Side faces use when same block above (sandstone) |
| `random` | any | Position-seeded tile selection |
| `repeat` | W×H | Pattern repeat |
| `fixed` | 1 | Always show one tile |

### Other fields
- `connect`: `tile` (default, texture match), `block`, `state`
- `faces`: `all`, `top`, `sides`, `north/south/east/west`
- `blocks`: restrict to specific blocks (by name, state, or tag like `#ores/*`)
- `heights`, `biomes`, `weights`, `symmetry`, `inner_seams`

### Models as Tiles
- If `models/block/<tile>.json` exists alongside the CTM tile, the tile redraws the whole block
- A `.ctm` can mix pictures and models in its `tiles` list
- `hides_neighbors = false` to avoid see-through gaps in custom models

## Resource Pack Architecture
### Layout
```
resourcepacks/
  my_pack/
    pack.cfg              # name + description (optional)
    base/
      textures/block/stone.png
      lang/en_us.lang
    examplemod/
      textures/item/example_item.png
```
- A pack's files live in `<namespace>/` folders matching mod IDs
- Stacking: "Selected" list, top wins
- F3+T reloads without restart

### pack.mcmeta (Minecraft standard, used by minecraft_assets crate)
Pumpkin uses its own config format (TOML `pumpkin.toml`), not pack.mcmeta.
Weft's pack.cfg has `name =` and `description =` only.
Minecraft's pack.mcmeta has `pack_format` integer and `description`.
Different ecosystems: Weft ≈ simplified, Minecraft's pack.mcmeta is the canonical format.

## Block Models (Java Edition format)
File: `assets/models/block/<block_name>.json`, exported from Blockbench

### JSON structure
```json
{
  "parent": "block/cube_all",  // or "block/cube", "block/cube_column", "block/cube_bottom_top", or another model path
  "textures": {
    "all": "block/stone",
    "side": "block/stone_side",
    "top": "block/stone_top",
    "bottom": "block/stone_bottom"
  },
  "elements": [
    {
      "from": [0, 0, 0],
      "to": [16, 16, 16],
      "rotation": { "angle": 0, "axis": "y", "origin": [0, 0, 0] },
      "faces": {
        "north":  { "texture": "#side",   "uv": [0, 0, 16, 16] },
        "south":  { "texture": "#side",   "uv": [0, 0, 16, 16] },
        "east":   { "texture": "#side",   "uv": [0, 0, 16, 16] },
        "west":   { "texture": "#side",   "uv": [0, 0, 16, 16] },
        "up":     { "texture": "#top",    "uv": [0, 0, 16, 16] },
        "down":   { "texture": "#bottom", "uv": [0, 0, 16, 16] }
      }
    }
  ]
}
```

### Key concepts
- `from/to`: 0-16 pixels, 16 pixels per block
- `#name`: refers to a `textures` entry; if no texture sets for a face, shows block's own face texture
- UV fractions supported for sub-pixel resolution on larger atlas textures
- Cube parents: `cube_all` (1 texture all faces), `cube` (separate 6 faces), `cube_column` (side+top/bottom), `cube_bottom_top`

## Bevy 0.20 Translation Table
| Weft Concept | Bevy 0.20 Equivalent | Notes |
|---|---|---|
| Bindless texture table | `Handle<Image>` + `TextureAtlas` | Bevy uses handle-based, Weft uses index-based |
| 16×16 individual textures | `TextureAtlasLayout` from `Image` grid | Load N textures, combine into one atlas `Handle<Image>` |
| `png` crate for decode | `Image::from_buffer` or `AssetServer<TexturesPath>` | Bevy has built-in PNG decode |
| `.anim` files | `AnimationFrame` / `Animation<Handle<Image>>` resources | Bevy animation system or manual timer |
| `_e.png` emission | N/A (use `StandardMaterial::emissive` per voxel, or custom shader) | Not in `StandardMaterial` built-in; custom PBR shader needed |
| `_overlay.png` tint | `StandardMaterial::base_color tint` overlay | Bevy doesn't have per-face overlay; use 2 materials or shader |
| `.ctm` connected textures | N/A | Not available in Bevy. Custom shader or pre-generated variant textures |
| `StandardMaterial::from(color)` | `StandardMaterial` with `base_texture: Some(handle)` + `texture_atlas_layout` | Current warren-game migration target |
| `Mesh3d(Cuboid)` per voxel | `Mesh3d(Cuboid)` + `MeshMaterial3d` with `TextureAtlasLayout` per face | Atlas UV per voxel instance |
| Resource pack F3+T reload | Bevy `AssetServer` + `AddAssetLoader` with `Reload` | Bevy supports hot-reload via asset system |
| `models/block/*.json` parsing | Use `minecraft_assets` crate or `serde_json` + custom structs | See texture-loader-crates.md |
| Face UV mapping on Cuboid | `Mesh3d` with custom `Mesh3d` from `gltf` or procedural UV gen | Bevy Cuboid doesn't natively support per-face UVs from a model JSON — need custom `Mesh` builder |

## Key Differences Weft vs Bevy
- Weft: each texture is a separate GPU texture indexed by `u32` → no atlas needed, no UV math. Bevy: `TextureAtlasLayout` needed for multiple faces from one image.
- Weft: no vertex buffers (shaders pull from storage) → custom architecture. Bevy: standard vertex buffer + mesh → use `Mesh3d(Cuboid)`.
- Weft's CTM is a texture-level feature. Bevy would need a similar custom shader or generation of variant textures at load time.
- Weft's emission (`_e`, `_s`) requires a PBR material extension. Bevy's `StandardMaterial` has no emission field in 0.20 — use `MeshStandardMaterial` with `emissive` component or bevy_pbr custom shader.
- Weft resource packs use `pack.cfg` (name+description only); Minecraft uses `pack.mcmeta` with `pack_format` version.

## Key Cargo Dependencies in Weft
| Crate | Version | Purpose | Bevy equivalent |
|---|---|---|---|
| `ash` | 0.38 | Vulkan API | N/A (Bevy abstracts) |
| `gpu-allocator` | 0.28 | GPU memory management | Bevy internal |
| `naga` | 30 | WGSL compile → SPIR-V/shaders | `bevy_render` |
| `glam` | 0.34 | Math (Vec3, Quat, Mat4) | `bevy Math` (prelude::Vec3 etc.) |
| `png` | 0.18 | PNG decode | Bevy `image` (built-in) |
| `zip` | 2 | ZIP resource packs | Not needed for folder layout |
| `serde_json` | — | JSON blockstate/model parsing | Use in warren-game for model parsing |

## MVP Scope Decision for warren-game
- **In scope**: Texture atlas from 16×16 PNGs, `Mesh3d` with `StandardMaterial{base_texture}`, `TextureAtlasLayout` for 6-face UV selection, block model JSON parsing via `serde_json` or `minecraft_assets` for cube_all/cube/cube_column
- **Out of scope** (future phases): CTM, animation, emission, overlays, connected textures, shader packs, LabPBR material
- **Texture atlas strategy**: Load all PNGs in `assets/textures/block/`, use `bevy::render::texture::TextureAtlasLayout` to compute UV regions, combine into single `Handle<Image>` (64×64 or larger grid) or keep individual images and index by handle
