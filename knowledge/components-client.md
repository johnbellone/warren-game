---
type: Architecture
title: Client Component (Tauri)
description: "The Tauri desktop application wraps the Bevy engine and Mantine UI, providing desktop integration and IPC bridge."
tags: [tauri, desktop, client, ipc]
generated: { by: agent/mcp, at: "2026-11-01T17:29:29Z" }
status: stable
---

## Tauri Desktop Application
The client/ directory contains the Tauri desktop application configuration.
## Role
- Wrap Bevy 3D scene and Mantine React UI in native desktop window
- Provide IPC bridge between Mantine UI (JS) and Bevy engine (Rust)
- Commands: start/stop server, create/destroy scene, sync state
## Tauri v2
Uses Tauri 2.x for multi-window and webview support.
Bevy window and Mantine webview are separate (MVP).
For later: single-window with Bevy rendering via canvas.
