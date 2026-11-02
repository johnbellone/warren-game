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
