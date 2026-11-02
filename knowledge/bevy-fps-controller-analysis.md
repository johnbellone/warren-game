---
type: Architecture
title: bevy_fps_controller — Source-Engine-Inspired FPS Movement for Bevy
description: "Source-engine-style FPS movement plugin for Bevy: air strafing, bunny hopping, slope walking, crouch/sprint, noclip. Supports both Rapier and Avian physics backends via feature flags. Reference for avatar movement in collaborative voxel editors."
tags: [analysis, bevy-fps-controller, movement, fps, bevy, rapier, avian, avatar, player-control]
generated: { by: agent/cli, at: "2026-11-02T21:14:39Z" }
status: stable
---

## Overview

bevy_fps_controller is a Source-engine-inspired first-person player movement plugin for Bevy. It supports both Rapier and Avian physics backends via Cargo feature flags.

## Features

| Feature | Detail |
|---------|--------|
| Air strafing | Classic FPS strafing while airborne |
| Bunny hopping | Jump while held for momentum preservation |
| Sloped ground movement | Walk along angled surfaces |
| Crouching | Crouch lowers eye height and prevents falling off ledges (Rapier only) |
| Sprinting | Fast movement mode |
| Noclip mode | Camera movement through walls (debug mode) |
| Step clearing | Instantly clear small ledges (Rapier only) |
| Configurable settings | All parameters are configurable |

## Usage

```bash
# Rapier backend
cargo run --release --features rapier --example minimal_rapier

# Avian backend
cargo run --release --features avian --example minimal_avian
```

## Relevance to warren-game

Not in MVP scope. Highly relevant when warren-game adds character or avatar movement for multiplayer collaboration (users "walking around" the voxel scene). The dual Rapier/Avian feature flag pattern is a good design reference for supporting both physics backends.

Source: https://github.com/qhdwight/voxel-game-rs also uses this crate.

# Related Concepts
- [Avian — ECS-Driven 2D/3D Physics Engine for Bevy](avian-analysis.md): bevy_fps_controller supports Avian via --features avian. Uses Avian for character physics in FPS movement.
- [Rapier — Multi-Platform Physics Engine (Deno/Rust)](rapier-analysis.md): bevy_fps_controller also supports Rapier via --features rapier (uses bevy-rapier wrapper, not Rapier directly).
