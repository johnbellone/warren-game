---
type: Architecture
title: Collaboration Architecture
description: "Server-authoritative collaborative editing over WebSocket with broadcast fan-out, last-write-wins conflict resolution, and presence-aware roster system."
tags: [architecture, collaboration, protocol, websocket, multiplayer]
generated: { by: agent/mcp, at: "2026-11-01T17:28:52Z" }
status: stable
---

## Architecture Overview
Server-authoritative model. A central WebSocket server maintains authoritative voxel state and broadcasts all changes to connected clients.
## Protocol Messages
Client to Server: Join { name, color_rgb }, VoxelEdit { pos, color, erase }, Heartbeat, Leave
Server to Client: Welcome { client_id, snapshot, roster }, Roster { users }, VoxelEdit { client_id, pos, color, erase }, HeartbeatAck, Error { message }
## Concurrency
- tokio async runtime
- tokio::sync::Mutex<ServerState> shared state
- tokio::sync::broadcast channel (capacity 4096) for fan-out
- Per-client task with tokio::select! loop
## Conflict Resolution
- Last-write-wins per voxel position (MVP)
- Server is single source of truth
- Future: per-voxel version counter, CRDT merge
## Roster
- PeerInfo: id, name, color_rgb, last_active_ms
- Roster broadcast on join and leave
- Client displays presence indicators
