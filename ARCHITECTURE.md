# Warren Game Editor Architecture

This document describes the architecture of the Warren Game Editor, a Tauri desktop application that embeds a Bevy engine for live game editing with multiplayer collaborative features.

## Overview

The Warren Game Editor combines:
1. **Tauri Desktop Application** - Cross-platform desktop app with web frontend
2. **Mantine 9 UI Framework** - For responsive and modern user interface 
3. **Bevy Engine** - Embedded game engine for live editing functionality
4. **Multiplayer Game Server** - Supports collaborative asset creation

## Project Structure

```
warren-game/
├── src-tauri/            # Tauri application and Rust backend
│   └── tauri.conf.json   # Tauri configuration
├── src/web/              # Bun web frontend (React + Mantine 9)
├── bevy/                 # Bevy engine integration
│   └── Cargo.toml        # Bevy dependencies
├── server/               # Game server with WASM support
│   ├── Cargo.toml        # Server dependencies  
│   └── README.md         # Server details
├── package.json          # Node.js/Bun dependencies
└── Cargo.toml            # Workspace manifest
```

## Key Components

### 1. Tauri Backend (`src-tauri/`)
- Rust-based desktop application framework
- System integration and file operations
- Communication bridge to web frontend
- Process management for Bevy engine
- WebSocket server for multiplayer connections

### 2. Web Frontend (`src/web/`)
- Bun-based React application
- Mantine 9 UI components
- Live preview controls
- Project configuration interface
- Multiplayer collaboration tools

### 3. Bevy Integration (`bevy/`)
- Embedded game engine runtime
- Asset loading and management
- Real-time rendering capabilities
- Communication with Tauri backend
- Live editing synchronization

### 4. Game Server (`server/`)
- Multiplayer support for collaborative editing
- Asset versioning and conflict resolution
- WebSocket communication protocol
- Real-time state synchronization

## Multiplayer Collaboration Features

The core requirement is to allow two or more users to simultaneously create and edit assets on the same server. This requires:

### 1. **Real-time Synchronization**
- Live updates of asset changes across all connected clients
- WebSockets for low-latency communication
- Conflict detection and resolution algorithms

### 2. **Asset Management System**  
- Version control for game assets
- Locking mechanisms for concurrent editing
- Merge strategies for conflicting changes
- Asset state tracking (created, modified, saved)

### 3. **Concurrency Control**
- User identification and session management
- Read/write access controls
- Change tracking and notifications
- Rollback capabilities

### 4. **Collaboration Tools**
- User presence indicators
- Real-time editing cursors/selections
- Communication channels
- Shared workspace environment

## Integration Requirements

The key challenge is integrating these components for a seamless collaborative experience:

1. **Tauri ↔ Bevy Communication**: Real-time updates between desktop app and embedded engine
2. **Web Frontend ↔ Server**: WebSocket connections for multiplayer collaboration
3. **Server ↔ Bevy**: Asset synchronization back to the engine for preview
4. **User Management**: Session tracking across all components

## Development Workflow

1. **Frontend Development**: Edit React components in `src/web/` with collaborative features
2. **Backend Development**: Handle system operations and multiplayer in `src-tauri/`
3. **Bevy Integration**: Implement engine communication in `bevy/`
4. **Server Development**: Add multiplayer capabilities in `server/`
5. **Build Process**: 
   - Tauri bundles the React app with the Rust backend
   - The Bevy engine runs as a separate component within the desktop app
   - Server can be compiled both natively and for WASM

## Technical Implementation Details

### 1. Connection Protocol
- WebSocket connections to server for multiplayer
- JSON-based messaging protocol between components
- Authentication tokens for user identification
- Session management across all components

### 2. Asset Handling
- Assets stored as versioned files in a shared repository
- Real-time synchronization during collaborative editing
- Conflict resolution when multiple users modify same asset
- Backup and recovery mechanisms

### 3. Real-time Updates
- Push-based updates to connected clients
- Delta-based synchronization to minimize bandwidth
- Local caching for performance
- Offline capabilities with sync on reconnection

### 4. Security Considerations
- User authentication and authorization
- Access controls for different asset types  
- Data validation before save operations
- Session timeout mechanisms