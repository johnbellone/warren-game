# Warren Game Editor Knowledge Base

This is the knowledge base for the Warren Game Editor project. It contains documented architectural decisions, component descriptions, and implementation details.

## Core Components

1. **[Project Foundation](foundation-decision.md)** - Overview of the complete project structure
2. **[Client Component (Tauri)](components-client.md)** - Desktop application backend  
3. **[Editor Component (Web Interface)](components-editor.md)** - Web-based UI with Bun and Mantine 9
4. **[Game Component (Bevy)](components-game.md)** - Embedded game engine integration
5. **[Server Component (Multiplayer)](components-server.md)** - Multiplayer server with WASM support
6. **[Collaboration Architecture](collaboration-architecture.md)** - Multiplayer editing features

## Overview

The Warren Game Editor is a comprehensive development environment that combines:
- Tauri desktop application for cross-platform deployment
- Mantine 9 web interface for user experience
- Bevy game engine for live editing capabilities  
- Multiplayer server for collaborative asset creation

This architecture supports simultaneous editing by multiple users with real-time synchronization and conflict resolution.