# Warren Game Editor Architecture

This document describes the architecture of the Warren Game Editor, a Tauri desktop application that embeds a Bevy engine for live game editing.

## Overview

The Warren Game Editor combines:
1. **Tauri Desktop Application** - Cross-platform desktop app with web frontend
2. **Mantine 9 UI Framework** - For responsive and modern user interface 
3. **Bevy Engine** - Embedded game engine for live editing functionality

## Project Structure

```
warren-game/
├── src-tauri/            # Tauri application and Rust backend
│   └── tauri.conf.json   # Tauri configuration
├── src/web/              # Bun web frontend (React + Mantine 9)
├── bevy/                 # Bevy engine integration
│   ├── Cargo.toml        # Bevy dependencies
│   └── README.md         # Integration details
├── package.json          # Node.js/Bun dependencies
└── Cargo.toml            # Main application manifest
```

## Key Components

### 1. Tauri Backend (`src-tauri/`)
- Rust-based desktop application framework
- System integration and file operations
- Communication bridge to web frontend
- Process management for Bevy engine

### 2. Web Frontend (`src/web/`)
- Bun-based React application
- Mantine 9 UI components
- Live preview controls
- Project configuration interface

### 3. Bevy Integration (`bevy/`)
- Embedded game engine runtime
- Asset loading and management
- Real-time rendering capabilities
- Communication with Tauri backend

## Development Workflow

1. **Frontend Development**: Edit React components in `src/web/`
2. **Backend Development**: Handle system operations in `src-tauri/`
3. **Bevy Integration**: Manage engine interaction in `bevy/`
4. **Build Process**: 
   - Tauri bundles the React app with the Rust backend
   - The Bevy engine runs as a separate component within the application

## Integration Requirements

The key challenge is integrating these three components:
- Communication between Tauri's Rust backend and Bevy 
- File system access from web frontend to Bevy
- Real-time rendering updates
- Asset management between components