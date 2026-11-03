---
type: Architecture
title: bevy_meshem — Voxel Grid to Bevy Mesh Generation
description: bevy_meshem generates 3D Bevy Meshes from voxel grids using Naive or Culling algorithms. VoxelRegistry trait for custom voxel types. O(1) runtime mesh updates via MeshMD change journal. Smooth lighting (ambient occlusion baked into vertex colours). Adjacent chunk seam culling. Targets Bevy 0.18; warren-game uses Bevy 0.20 — port needed.
tags: [architecture, bevy, voxel, meshing, grid, cube, culling, ambient-occlusion, run-time-update, bevy-0.18, bevy-0.20-gap]
generated: { by: agent/cli, at: "2026-11-03T01:43:09Z" }
status: stable
---

## Overview

`bevy_meshem` (crate: `bevy_meshem`, repo: `Adamkob12/Meshem` / `Adamkob12/bevy_meshem`) is a Rust crate that generates 3D meshes from a grid of voxels for Bevy. It implements two voxel meshing algorithms (Naive and Culling) plus runtime mesh updates without full regeneration, smooth lighting (ambient occlusion), and adjacent chunk seam culling.

## Key API

### VoxelRegistry Trait

The core abstraction — implement this for your voxel data structure:

```rust
pub trait VoxelRegistry {
    type Voxel: Debug + Eq + PartialEq + Clone + Copy;

    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh>;
    // Returns None for air/empty, a Mesh for solid voxels

    fn is_covering(&self, voxel: &Self::Voxel, side: Face) -> bool;
    // Does this voxel block light from this direction? (occlusion test)

    fn get_center(&self) -> [f32; 3];
    fn get_voxel_dimensions(&self) -> [f32; 3];
    fn all_attributes(&self) -> Vec<MeshVertexAttribute>;
    // Position, Normal, UV, Color, etc.
}
```

### VoxelMesh Enum

```rust
pub enum VoxelMesh<T> {
    NormalCube(T),    // A standard cube
    CustomMesh(T),    // Arbitrary mesh (partially implemented, warns on unwrap)
    Null,             // Air / no geometry
}
```

### mesh_grid() — Primary Meshing Function

```rust
pub fn mesh_grid<T>(
    dims: Dimensions,              // (width, height, length)
    outer_layer: &[Face],         // Which outer faces to cull
    grid: &[T],                   // 1D voxel array
    reg: &impl VoxelRegistry,      // Your voxel registry
    meshing_algorithm: MeshingAlgorithm,
    smooth_lighting_params: Option<SmoothLightingParameters>,
) -> Option<(Mesh, MeshMD<T>)>
```

Returns a `Mesh` and `MeshMD<T>` (mesh metadata) for runtime tracking.

## Meshing Algorithms

| Algorithm | Vertex count (10³ grid) | Description |
|---|---|---|
| **Naive** | 24,000 | One cube per voxel, no culling |
| **Culling** | 2,400 | Skips hidden faces (Minecraft-style) |
| **Greedy** | Not yet | Deferred — too lossy for textured voxels |

## Runtime Mesh Updates (v0.2+)

`MeshMD<T>` tracks a change journal. No need to regenerate the entire mesh after a voxel edit.

```rust
pub enum VoxelChange {
    Added,       // Voxel was placed
    Broken,      // Voxel was removed
    CullFaces,   // Faces of this voxel should be hidden
    AddFaces,    // Faces of this voxel should be visible
}

// Log a change, then call update_mesh()
metadata.log(VoxelChange::Added, voxel_index, &new_voxel, neighbors);
update_mesh(&mut mesh, &mut metadata, &registry);
```

This applies to the `Mesh` in-place without regeneration.

## Smooth Lighting (v0.3+)

Static ambient occlusion baked into vertex colours during mesh generation. Each vertex gets a shadow value based on neighbouring voxel occlusion. No per-frame cost — computed once.

## Adjacent Chunk Culling

`introduce_adjacent_chunks()` fixes seams between two independently generated chunk meshes:
- Culls hidden vertices trapped between chunks
- Applies smooth lighting across the chunk boundary
- Takes the adjacent chunk's grid as a parameter, no shared state

## Crate Structure

```
src/
  lib.rs              -- VoxelRegistry trait, prelude, core enums
  meshem.rs           -- mesh_grid(), add_vertices_normal_cube()
  update.rs           -- update_mesh(), add_voxel_after_gen(), remove_voxel()
  face.rs             -- Face enum (Top/Bottom/Right/Left/Back/Forward), encoding
  adj.rs              -- introduce_adjacent_chunks()
  mesh_metadata.rs    -- MeshMD<T>, change journal
  pbs.rs              -- per-block shading data
  voxel_mesh.rs       -- generate_voxel_mesh() — default cube mesh with UV/normal/color
  util/
```

## Dependencies

| Dependency | Version | Purpose |
|---|---|---|
| `bevy` | `0.18.0` | Rendering, Mesh type |
| `rand` | `0.8.5` | (used in examples for terrain gen) |

## Bevy Compatibility

| bevy_meshem | Bevy Version |
|---|---|
| 0.5.x (latest) | **0.18** |
| 0.3.x | 0.12 |
| 0.2.x | 0.11.x |
| 0.1.1 | 0.11 |

**Warren-game uses Bevy 0.20 — not covered.** Porting would require adapting to Bevy 0.20's Mesh API changes.

## Related Projects by Author

| Project | Status | Purpose |
|---|---|---|
| `minecraft_bevy` | **Archived** | Minecraft clone using bevy_meshem; showcases features |
| `NovaCraft` | Active | Re-iteration of minecraft_bevy; possibly updated to newer Bevy |

`NovaCraft` may contain a Bevy 0.20+ port or be worth checking.

## Relevance to warren-game

`bevy_meshem` solves the exact problem warren-game faces: rendering a `VoxelWorld` grid as a single optimised Bevy `Mesh` with culling.

**Direct relevance:**
1. **VoxelRegistry** is the same pattern warren-game needs — implement it for `VoxelBlock`
2. **MeshMD + update_mesh()** provides O(1) runtime updates — critical for collaborative editing where voxels change constantly
3. **Face culling** reduces vertex count by 10x — a bare 64³ world goes from ~1.5M vertices (naive) to ~150K (culled)
4. **Adj chunk culling** enables a chunking strategy without shared-state coordination

**Compatibility gap:**
`bevy_meshem` 0.5.1 targets Bevy 0.18, warren-game uses 0.20. The `Mesh` type API in Bevy 0.20 has changed (vertex attributes moved to a new API). A port would be needed but should be tractable.

**Recommendation for MVP:** Start with the Naive algorithm to get voxel rendering working, then migrate to Culling + MeshMD for production.

# Related Concepts
- [Texture Loader Crate Analysis — minecraft_assets vs. Custom vs. Bevy Built-in](texture-loader-crates.md): bevy_meshem generates mesh geometry from a voxel grid; texture-loader-crates handles the texture atlas and model data that meshem applies as vertex attributes. Together, they form the full voxel rendering pipeline.
- [Editor Component (Mantine UI)](components-editor.md): bevy_meshem is the rendering layer for the editor's voxel data model. The Mantine UI triggers voxel edits that feed into meshem's VoxelChange journal and update_mesh().
