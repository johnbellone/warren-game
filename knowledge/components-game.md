---
type: Architecture
title: Game Component (Voxel World)
description: "The warren-game crate defines the voxel world data model: sparse HashMap storage, brush modes, and world bounds."
tags: [bevy, voxel, game, data-model]
generated: { by: agent/mcp, at: "2026-11-01T17:28:52Z" }
status: stable
---

## Voxel World
The warren-game crate defines the core voxel data model.
## Storage
- HashMap<[i32;3], Voxel> sparse storage, only occupied positions
- Bounded volume: [64,64,64] half-extents = 262,144 max voxels
## Voxel Type
pub struct Voxel { position: [i32;3], color: [f32;3], asset_type: AssetType }
## AssetType
MVP: Voxel only. Future: Model, Material, PhysicsProp, Script.
## Operations
place(), erase(), apply_edit(VoxelEdit), snapshot() -> Vec<Voxel>
