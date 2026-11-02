---
type: Architecture
title: Avian — ECS-Driven 2D/3D Physics Engine for Bevy
description: "Deep analysis of avianphysics/avian: ECS-native physics engine built in Rust for Bevy, backed by Parry collision detection; rigid bodies, joints, spatial queries, Transform interpolation, modular plugins, f32/f64 precision. MIT/Apache 2.0. Bevy 0.20 → avian3d 0.8."
tags: [analysis, avan, physics, bevy, rapier, parry, ecs, rigid-bodies, joints, sim]
generated: { by: agent/cli, at: "2026-11-02T21:14:39Z" }
status: stable
---

## Overview

Avian is a **first-party physics engine built inside Bevy** — not a wrapper. It provides 2D (`avian2d`) and 3D (`avian3d`) crates.

## Design Principles

- **ECS-native**: No separate physics world. Components and systems are first-class Bevy items.
- **Modular plugin architecture**: Parts can be replaced (e.g., collision backend).
- **Custom collision backends** supported (Parry is the default).
- **f32 / f64 precision** (f32 by default).

## Core Features

| Feature | Detail |
|---------|--------|
| Rigid bodies | Dynamic, kinematic, static; linear/angular velocity, forces, torques, damping, CCD, sleeping |
| Collision detection | Backed by Parry; configurable layers, density, material; scene colliders; collision events; filtering/modifying via collision hooks; manual contact queries |
| Joints & constraints | Fixed, distance, prismatic, revolute, spherical; custom joints via XPBD |
| Spatial queries | Raycast, shape cast, point projection, intersection test; component-based API; `SpatialQuery` parameter with filters |
| Transform interpolation | Interpolation and extrapolation for fixed timesteps |
| Debug rendering | Colliders, AABBs, contacts, joints, spatial queries |
| Scheduling | Fully configurable; high customizability |

## Integration Example

```rust
use avian3d::prelude::*;
use bevy::prelude::*;

App::new()
    .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
    .add_systems(Startup, setup)
    .run();
```

Spawn a rigid body by adding `RigidBody::Dynamic` or `RigidBody::Static` + `Collider` components:

```rust
commands.spawn((
    RigidBody::Static,
    Collider::cuboid(1.0, 1.0, 1.0),
    Mesh3d(meshes.add(Cuboid::from_length(1.0))),
));
```

## Version Compatibility

| Bevy | Avian |
|------|-------|
| 0.20 | 0.8 |
| 0.19 | 0.7 |
| 0.18 | 0.5–0.6 |
| 0.17 | 0.4 |
| 0.16 | 0.3 |
| 0.15 | 0.2 |

Bevy XPBD (predecessor) supported Bevy 0.10–0.14.

## Relevance to warren-game

- Currently **out of MVP scope** (MVP plan lists "Physics simulation" as a future feature).
- Relevant when voxel game requires: falling chunks, dynamic props, ragdoll characters, or vehicle support.
- Avian is the recommended Bevy integration layer over Rapier; bevy_fps_controller supports both Rapier and Avian via feature flags.

## License

MIT / Apache 2.0

# Related Concepts
- [Rapier — Multi-Platform Physics Engine (Deno/Rust)](rapier-analysis.md): Avian is the Bevy-native integration layer for Rapier. Uses Parry (Rapier's collision geometry crate) as its default collision backend.
