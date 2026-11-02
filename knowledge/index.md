# Warren Game Editor Knowledge Base

This is the knowledge base for the Warren Game Editor project. It contains documented architectural decisions, component descriptions, and implementation details.

## Core Components

* [Project Foundation](foundation-decision.md) - The Warren Game Editor uses a Cargo workspace with protocol, server, and game crates, plus a Tauri + Mantine editor frontend, targeting Bevy 0.20.
* [Client Component (Tauri)](components-client.md) - The Tauri desktop application wraps the Bevy engine and Mantine UI, providing desktop integration and IPC bridge.
* [Editor Component (Mantine UI)](components-editor.md) - The editor is a Mantine React UI embedded in a Tauri desktop window, providing color picker, tool selection, roster, and connection status.
* [Game Component (Voxel World)](components-game.md) - The warren-game crate defines the voxel world data model: sparse HashMap storage, brush modes, and world bounds.
* [Server Component (WebSocket)](components-server.md) - The warren-server crate provides a WebSocket server for collaborative voxel editing, using tokio + tokio-tungstenite with broadcast channel fan-out.
* [Collaboration Architecture](collaboration-architecture.md) - Server-authoritative collaborative editing over WebSocket with broadcast fan-out, last-write-wins conflict resolution, and presence-aware roster system.

## Overview

The Warren Game Editor is a comprehensive development environment that combines:
- Tauri desktop application for cross-platform deployment
- Mantine 9 web interface for user experience
- Bevy game engine for live editing capabilities  
- Multiplayer server for collaborative asset creation

This architecture supports simultaneous editing by multiple users with real-time synchronization and conflict resolution.
* [MVP Plan: Collaborative Voxel Asset Editor](mvp-plan.md) - Architecture and phased delivery plan for the Warren Game Editor MVP: two or more Tauri instances run a Bevy 3D scene and Mantine UI, connected to a shared WebSocket server for real-time collaborative voxel editing.
* [Weft Voxel Engine Analysis](weft-analysis.md) - Deep analysis of Weft's Vulkan rendering pipeline, 16x16 texture system, .anim/_e/overlay/ctm sidecar files, resource pack layout, Java block model JSON, and Bevy 0.20 translation for each concept.
* [Pumpkin Minecraft Server Analysis](pumpkin-analysis.md) - Analysis of NBT data model (pumpkin-nbt), chunk format (pumpkin-world), Java/Bedrock protocol, skin handling, TOML config, and WASM plugin host as reference for warren-game.
* [Texture Loader Crate Analysis](texture-loader-crates.md) - Comparison of minecraft_assets crate vs Bevy 0.20 built-in TextureAtlasLayout vs custom serde_json parser; recommendation for warren-game MVP.
* [Texture Loading Architecture](texture-loader-architecture.md) - Proposed module structure, data structures, Bevy system layout, and migration path for adding textured cubes to warren-game's voxel world.
* [aeronet Analysis](aeronet-analysis.md) - Deep analysis of aecsocket/aeronet: ECS-native networking, swappable IO layers (channel/WebSocket/WebTransport/Iroh/Steam), layered transport stack, and warren-game migration via aeronet_websocket.
* [bevy_replicon Analysis](bevy-replicon-analysis.md) - Server-authoritative ECS world replication, remote events/triggers, authorization, client-side visibility, no builtin IO with 5 messaging backends, Bevy 0.20 via 0.45.
* [lightyear Analysis](lightyear-analysis.md) - Bevy multiplayer framework layering bevy_replicon on aeronet with prediction/rollback/interpolation/bandwidth; Bevy 0.20 via 0.31; overkill for the editor.
* [matchbox Analysis](matchbox-analysis.md) - P2P WebRTC networking for wasm+native, signaling server, ggrs/leafwing; bevy_matchbox tops out at Bevy 0.17, incompatible with warren-game's Bevy 0.20.
* [Networking Cross-Crate Comparison](networking-comparison.md) - Comparison matrix of all 4 crates, lightyear=bevy_replicon+aeronet relationship, and warren-game recommendation (aeronet+bevy_replicon MVP, defer lightyear, reject P2P matchbox).
