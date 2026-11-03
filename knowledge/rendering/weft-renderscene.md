---
type: Architecture
title: "Warren-Game Bevy 0.20 Rendering & Animation Decisions"
description: "Verified grid-blink fix (single textured plane), Bevy 0.20 material/texture API corrections, asset path resolution, and character auto-walk animation design for warren-game."
tags: [bevy, grid, texture, animation, character, decisions, cargo]
generated: { by: agent/cli, at: "2026-11-03T15:44:23Z" }
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

# Related Concepts
- [Weft Voxel Engine — Texture Atlas & Resource Pack Analysis](../weft-analysis.md): Corrects and extends the weft-analysis Bevy 0.20 translation table with actual verified API (base_color_texture, emissive LinearRgba, single-plane textured grid, auto-walk animation)
- [Game Component (Voxel World)](../components-game.md): Concrete implementation of the playable character entity and environment rendering described by components-game
