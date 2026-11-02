---
type: Decision
title: Project Foundation
description: "The Warren Game Editor uses a Cargo workspace with protocol, server, and game crates, plus a Tauri + Mantine editor frontend, targeting Bevy 0.20."
tags: [architecture, foundation, cargo, bevy, tauri]
generated: { by: agent/mcp, at: "2026-11-01T17:29:29Z" }
status: stable
---

## Project Structure
warren-game/ (Cargo workspace)
  protocol/ - Shared wire types: ClientMsg, ServerMsg, Voxel, PeerInfo
  server/ - WebSocket server (tokio + tokio-tungstenite)
  game/ - Voxel world data model (Bevy integration later)
  client/ - Tauri desktop app configuration
  editor/ - Mantine React UI (TSX)
  docs/ - Documentation including MVP_PLAN.md
  knowledge/ - OKF knowledge base
## Key Decisions
- Bevy 0.20 latest stable. bevy_voxel is a reserved placeholder; build voxel rendering ourselves using Mesh3d cuboids.
- protocol crate is single source of truth for wire types
- Server authoritative: server validates and broadcasts all edits
- Sparse voxel storage: HashMap for simplicity, chunk-based later
- Tauri + Mantine: desktop UI via Tauri, web UI via Mantine React
