# Client Component (Tauri Application)

## Description
The client component is the Tauri-based desktop application that serves as the main interface for the Warren Game Editor. It provides system integration, communication with other components, and process management.

## Key Functions
- System integration and file operations
- Communication bridge to web frontend  
- Process management for game engine
- WebSocket server for multiplayer connections
- Cross-platform deployment capabilities

## Technical Details
- Built with Rust and Tauri framework
- Uses tauri.conf.json for configuration
- Supports multi-platform deployment (Windows, macOS, Linux)
- Integrated with the game engine for live preview functionality

## Integration Points
Connects to:
- Web frontend through HTTP/WebSocket communication
- Game component for real-time rendering updates 
- Server component for multiplayer coordination