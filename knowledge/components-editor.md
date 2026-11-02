---
type: Architecture
title: Editor Component (Mantine UI)
description: "The editor is a Mantine React UI embedded in a Tauri desktop window, providing color picker, tool selection, roster, and connection status."
tags: [editor, mantine, react, tauri, ui]
generated: { by: agent/mcp, at: "2026-11-01T17:29:29Z" }
status: stable
---

## Editor UI
The editor/ directory contains the Mantine React UI within a Tauri desktop window.
## Components (MVP)
- Connection Panel: start/stop server, enter name, choose avatar color
- Color Picker: select current voxel color
- Toolbar: Place mode / Erase mode toggle
- Roster Panel: display online users with avatar colors
- Connection Status: indicator showing connection state
## Integration
Runs as a webview inside Tauri. Communicates with Rust backend via Tauri IPC.
Bevy rendering in separate native window (MVP) or same window (later).
