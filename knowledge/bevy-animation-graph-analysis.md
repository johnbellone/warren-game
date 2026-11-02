---
type: Architecture
title: bevy_animation_graph — Animation Graphs and State Machines for Bevy
description: "Animation graphs as Bevy assets with a visual editor, state machines, IK, 2D blend spaces, ragdoll simulation, and custom node support. Two crates: library + editor. Separate from bevy_animation v0.14+ graphs."
tags: [analysis, bevy-animation-graph, animation, state-machine, ik, blend-space, ragdoll, bevy, editor]
generated: { by: agent/cli, at: "2026-11-02T21:11:54Z" }
status: stable
---

## Overview

bevy_animation_graph is a complete animation system for Bevy that supersedes most of `bevy_animation`. It provides animation graphs, state machines, and a graphical editor — all as Bevy assets.

**Note**: This is separate from the animation graphs introduced in `bevy_animation` v0.14. This crate predates and is independent of that.

## Architecture

Two crates:
- `bevy_animation_graph` — runtime library
- `bevy_animation_graph_editor` — visual graph editor (Bevy app)

### Animation Graphs
- Assets — load from files or create in code
- Nestable — a graph can be a node inside another graph
- Output caching — avoids redundant computation
- Custom nodes — register Rust-defined nodes via Bevy type registry

### Animation State Machines
- Embedded as nodes within graphs
- Each state plays its own animation graph
- Transitions also have their own animation graph — can query source and target state graphs

### Built-in Nodes
- Animation chaining (play-on-complete)
- Two-bone inverse kinematics (IK)
- Looping
- Linear blending (bone space)
- 2D blend space
- Mirror about YZ plane
- Animation clip playback
- Bone-mask rotation application
- Speed up/down
- Graph node (nested graph reference)
- State machine node
- Arithmetic nodes (operations on f32, Vec3, Quat, etc.)

### Ragdoll Support
- Visual ragdoll editing
- Partial ragdolls (some bones simulated, others kinematically driven by target pose)

## Usage

```toml
[dependencies]
bevy_animation_graph = "0.9"  # Note: 0.9 introduces breaking changes from 0.8
```

## Planned Features

- Ragdoll "pose following" modes:
  - **Absolute matching** — simulated bone tracks world-space position/rotation of target bone via spring force
  - **Relative matching** — simulated bone tracks target's position/rotation relative to parent bone
  - Both may need joint motor support on Avian

## Relevance to warren-game

Not in MVP scope. Relevant when the voxel editor supports character models or animated assets. The graph-asset pattern is a useful pattern for any data-driven visual system.
