---
type: Decision
title: "MVP Plan: Collaborative Voxel Asset Editor"
description: "Architecture and phased delivery plan for the Warren Game Editor MVP: two or more Tauri instances run a Bevy 3D scene and Mantine UI, connected to a shared WebSocket server for real-time collaborative voxel editing."
tags: [mvp, architecture, collaboration, voxel, bevy, tauri, protocol]
generated: { by: agent/mcp, at: "2026-11-01T17:15:35Z" }
status: draft
---

## MVP Goal

Two or more Tauri desktop instances run simultaneously. Each embeds a Bevy 3D scene
and a Mantine React UI panel. They connect to a shared WebSocket server and
real-time edit a shared voxel scene together.

## Component Roles

`server` (Rust binary): WebSocket server, authoritative voxel state
`game` (Rust + Bevy): Voxel rendering, camera, brush input, WebSocket client
`client` (Rust + Tauri): Desktop app, launches Bevy + Mantine, bridges IPC
`editor` (TSX/React/Mantine): Color picker, toolbar, online users panel

## Scope In (MVP)

- Voxel editing: place, recolor, erase cubes
- Sparse storage: `HashMap<[i32;3], Voxel>`, fixed bounds [64,64,64]
- Server-authoritative, last-write-wins conflict resolution
- Presence/roster with avatar colors
- Bevy rendering: Mesh3d cuboid per voxel, PbrMaterial, orbit camera
- Mantine UI: color picker, brush tool, roster panel, connection status
- Tauri desktop app wrapping Bevy + Mantine

## Scope Out (Future)

- Chunk meshing, model import (OBJ/GLTF), physics, scripts
- Asset versioning/undo/redo
- Persistence, authentication, WASM server

## Key Decisions

`bevy_voxel` on crates.io is a reserved placeholder (0.0.1, empty). We build a
minimal custom voxel renderer with Bevy`s Mesh3d cuboid meshing.

Server authoritative, last-write-wins per voxel.
JSON over WebSocket for protocol.
Sparse HashMap for voxel storage.
MVP demo runs as standalone binaries before Tauri integration.

## Phases

1. **Foundation**: fix workspace, update Bevy to 0.20, add `warren-protocol` crate, `warren-server` binary
2. **Server**: WebSocket server, `VoxelWorld`, join/leave/heartbeat/voxel edit
3. **Bevy Voxel Rendering**: mesh cubes, camera, brush input, click raycast
4. **Client to Server**: WebSocket client, send remote edits, apply incoming
5. **Mantine UI**: color picker, toolbar, roster, connection status
6. **Tauri Integration**: wrap all in Tauri desktop app
