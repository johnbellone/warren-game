# Game Component (Bevy Engine)

## Description
The game component is the embedded Bevy game engine integration that provides real-time rendering and editing capabilities within the application.

## Key Functions
- Embedded Bevy engine runtime
- Asset loading and management 
- Real-time rendering capabilities
- Communication with Tauri backend
- Live editing synchronization

## Technical Details
- Uses Bevy 0.13 engine version
- Integrated with Tauri desktop application
- Supports live preview of edits
- Handles asset processing in real-time

## Integration Points
Connects to:
- Client component for system operations
- Editor component for user interaction  
- Server component for collaborative updates