---
type: Architecture
title: Server Component (WebSocket)
description: "The warren-server crate provides a WebSocket server for collaborative voxel editing, using tokio + tokio-tungstenite with broadcast channel fan-out."
tags: [server, websocket, tokio, multiplayer]
generated: { by: agent/mcp, at: "2026-11-01T17:29:29Z" }
status: stable
---

## Server Architecture
The warren-server crate runs a WebSocket server for collaborative voxel editing.
## Design
- tokio async runtime, tokio-tungstenite for WebSocket
- tokio::sync::broadcast (capacity 4096) for fan-out
- Per-client task with tokio::select! for incoming/outgoing
- Server state: Mutex<ServerState>
## Message Flow
1. Client connects, receives Welcome with full voxel snapshot + roster
2. Client sends Join, server adds peer, broadcasts Roster
3. Client sends VoxelEdit, server applies, broadcasts VoxelEdit
4. Client disconnects, server removes peer, broadcasts Roster
## Entry Point
cargo run -p warren-server -- --port 3000
cargo run -p warren-server -- --host 0.0.0.0 --port 3000
## MVP Limitations
- No persistence to disk
- No authentication
- Single shared world (no multi-scene)
