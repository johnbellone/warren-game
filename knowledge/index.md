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
* [Avian — ECS-Driven 2D/3D Physics Engine for Bevy](avian-analysis.md) - Deep analysis of avianphysics/avian: ECS-native physics engine built in Rust for Bevy, backed by Parry collision detection; rigid bodies, joints, spatial queries, Transform interpolation, modular plugins, f32/f64 precision. MIT/Apache 2.0. Bevy 0.20 → avian3d 0.8.
* [bevy_water — Dynamic Ocean Material for Bevy](bevy-water-analysis.md) - Dynamic water shader with wave displacement, get_wave_point for object placement, tileable, normals for lighting. WebGPU/WebGL2 via WASM. Bevy 0.20 compatible.
* [bevy_hanabi — GPU Particle System Plugin for Bevy](bevy-hanabi-analysis.md) - High-performance GPU particle system using compute shaders. EffectAsset with Module/expression language. Supports spawning, gravity, lifetime, and gradient colors. WebGPU-only for wasm. Bevy 0.20.
* [bevy_animation_graph — Animation Graphs and State Machines for Bevy](bevy-animation-graph-analysis.md) - Animation graphs as Bevy assets with a visual editor, state machines, IK, 2D blend spaces, ragdoll simulation, and custom node support. Two crates: library + editor. Separate from bevy_animation v0.14+ graphs.
* [bevy_console — Half-Life-Inspired Console for Bevy](bevy-console-analysis.md) - Command console for Bevy with clap-based parsing, history, tab completion, ANSI colors, and Bevy log capture. WASM compatible. Half-Life-inspired design.
* [Rapier — Multi-Platform Physics Engine (Deno/Rust)](rapier-analysis.md) - Multi-platform Rust physics engine by Dimforge: rigid bodies, force-based joints, contact events/sensors, scene queries, SIMD+parallelism, WASM via @dimforge NPM, optional cross-platform determinism. Backing engine for Avian.
* [bevy_fps_controller — Source-Engine-Inspired FPS Movement for Bevy](bevy-fps-controller-analysis.md) - Source-engine-style FPS movement plugin for Bevy: air strafing, bunny hopping, slope walking, crouch/sprint, noclip. Supports both Rapier and Avian physics backends via feature flags. Reference for avatar movement in collaborative voxel editors.
* [Bevy Settings — SettingsGroup, SaveSettingsDeferred, and Persistent Config](bevy-settings-example.md) - Bevy SettingsPlugin pattern: SettingsGroup derive merges resource sections into one config file. SaveSettingsDeferred for debounced autosave. SaveSettingsSync::IfChanged before exit. Reverse-Domain-Name namespace.
* [Bevy Plugin — Custom Plugin Registration Pattern](bevy-plugin-example.md) - Bevy Plugin trait: implement build(&self, &mut App) to register scoped components, resources, and systems. Configuration via struct fields. Composable via add_plugins(). The foundation of Bevy app composition.
* [Bevy Logs — LogPlugin, Log Levels, and once! Spam Prevention](bevy-logs-example.md) - Bevy logging: LogPlugin with configurable level and per-crate filter. trace/debug/info/warn/error levels. once! variants log once per call site for per-frame systems. RUST_LOG env var for runtime control via tracing-subscriber.
