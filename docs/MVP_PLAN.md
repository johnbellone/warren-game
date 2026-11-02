# Warren Game Editor — MVP Plan: Collaborative Voxel Asset Editor

## 1. Target

Two or more Tauri desktop instances run simultaneously. Each embeds a Bevy 3D scene
and a Mantine React UI panel. They connect to a shared WebSocket server and
real-time edit a shared voxel scene together.

## 2. Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Tauri: Alice│     │ Tauri: Bob  │     │ Tauri: Carol│
│ ┌─────────┐ │     │ ┌─────────┐ │     │ ┌─────────┐ │
│ │ Bevy 3D │ │     │ │ Bevy 3D │ │     │ │ Bevy 3D │ │
│ │ scene   │ │     │ │ scene   │ │     │ │ scene   │ │
│ ├─────────┤ │     │ ├─────────┤ │     │ ├─────────┤ │
│ │ Mantine │ │     │ │ Mantine │ │     │ │ Mantine │ │
│ │ UI      │ │     │ │ UI      │ │     │ │ UI      │ │
│ └─────────┘ │     │ └─────────┘ │     │ └─────────┘ │
│  ws://      │     │  ws://      │     │  ws://      │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                     │
       └───────────────────┼─────────────────────┘
                           │
                     ┌─────┴─────┐
                     │  Server   │
                     │ (binary)  │
                     │ WebSocket │
                     │ authoritative
                     │ voxel state
                     └───────────┘
```

### Component Roles

| Component       | Crate/Module      | Language    | Responsibility                                |
|-----------------|-------------------|-------------|-----------------------------------------------|
| `server`        | warren-server     | Rust        | WebSocket server, authoritative voxel state   |
| `game`          | warren-game       | Rust (Bevy) | Voxel rendering, camera, brush input, WS client|
| `client`        | warren-client     | Rust (Tauri)| Tauri window lifecycle, IPC bridge to Bevy+Mantine |
| `editor`        | —                 | TSX/React   | Mantine UI: palette, tools, roster, connection|

### MVP Scope: What's In

- Voxel editing: place, recolor, erase a single cube
- Sparse voxel storage: `HashMap<[i32;3], Voxel>` in a bounded volume
- Server-authoritative conflict resolution: last-write-wins per position
- Presence/roster: who's connected, their avatar color
- Bevy 3D rendering of voxels as `Mesh3d` cubes with `PbrMaterial`
- Basic camera navigation (orbit + WASD movement)
- Mantine UI: color picker, toolbar (place/erase mode), online peers list
- WebSocket protocol over JSON
- Tauri desktop app that runs Bevy + Mantine + connects to server

### MVP Scope: What's Not (Future)

- Chunk-based meshing / greedy meshing
- Model import (OBJ/GLTF)
- Physics simulation
- Script blocks
- Asset versioning / undo/redo
- Persistence to disk
- Authentication / user accounts
- WASM server build
- Network error recovery / reconnection
- Mobile / web build

## 3. Data Model

### 3.1 Core Types

```rust
// server/src/voxel.rs
pub struct Voxel {
    pub position: [i32; 3],  // world-space position
    pub color: [f32; 3],     // RGB 0.0..1.0
    pub asset_type: AssetType,
}

pub enum AssetType {
    Voxel,       // MVP: implement this fully
    Model,       // reference to 3D model, MVP: type only
    Material,    // PBR material, MVP: type only
    PhysicsProp, // physics property, MVP: type only
    Script,      // script block, MVP: type only
}

pub struct VoxelWorld {
    voxels: HashMap<[i32; 3], Voxel>,
    bounds: [i32; 3], // e.g. [64, 64, 64] = 262,144 max voxels
}
```

### 3.2 Protocol Types

```rust
// server/src/protocol.rs
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    Join { name: String, color_rgb: [f32; 3] },
    VoxelEdit {
        position: [i32; 3],
        color: [f32; 3],
        erase: bool,
    },
    Heartbeat,
    Leave,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMsg {
    // Sent on connect
    Welcome {
        client_id: String,
        snapshot: Vec<Voxel>,   // full world state
        roster: Vec<PeerInfo>,
    },
    // Broadcast on join/leave
    Roster { users: Vec<PeerInfo> },
    // Broadcast per voxel edit
    VoxelEdit {
        client_id: String,
        position: [i32; 3],
        color: [f32; 3],
        erase: bool,
        timestamp: u64,  // nanoseconds, for ordering
    },
    // Server heartbeat ack
    HeartbeatAck,
    Error { message: String },
}

pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub color_rgb: [f32; 3],
    pub last_active_ms: u64,
}
```

### 3.3 Server State

```rust
pub struct Server {
    world: VoxelWorld,
    peers: HashMap<String, PeerInfo>,  // client_id → PeerInfo
    broadcast: tokio::sync::broadcast::Sender<String>,  // JSON messages
}
```

## 4. Bevy Rendering

### Rendering Strategy

For MVP: each voxel is a `Mesh3d` with a `Cuboid` mesh + `PbrMaterial` instance.
No chunk meshing, no LOD. This is sufficient for hundreds to a few thousand
voxels, which is the MVP target.

For later: switch to `voxel-framework` or `bevy_voxel_world` for chunk-based
greedy meshing.

### Brush / Camera

- `Camera3d` with `OrbitCamera` + WASD movement (bevy_xpbd for movement)
- Left-click: place voxel at ray target (nearest voxel face)
- Right-click: erase voxel at ray target
- Brush size: 1×1×1 for MVP

### Bevy Resource

```rust
#[derive(Resource)]
pub struct VoxelEditor {
    voxels: HashMap<IVec3, Voxel>,
   brush_mode: BrushMode,
    current_color: [f32; 3],
    ws_client: WebSocketClient,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BrushMode { Place, Erase }
```

## 5. Server Design

### Implementation

- `tokio` async runtime
- `tokio::net::TcpListener` + `tokio-tungstenite` for WebSocket
- One `tokio::task` per connection
- `tokio::sync::broadcast` for fan-out to all connected clients
- Server is authoritative: all edits go through the server, server assigns
  timestamps and broadcasts

### Concurrency

```
Connection task:
  1. Accept WS connection
  2. Read Join message → add peer to roster
  3. Send Welcome { snapshot, roster }
  4. Loop: read ClientMsg → process → broadcast to all
  5. On disconnect: remove peer, broadcast updated Roster

Broadcast receiver (one per connection):
  1. Read from broadcast channel
  2. Send to this client

Shared state:
  world: Mutex<VoxelWorld>
  peers: Mutex<HashMap<String, PeerInfo>>
```

### Conflict Resolution (MVP)

- Server assigns a monotonically increasing `edit_seq` per voxel per session
- Last-write-wins: if two clients edit the same voxel in the same frame,
  the one the server processes last wins
- No per-voxel version tracking in MVP. For later: add version field.

## 6. Tauri + Bevy Integration

### Pattern

Tauri v2 supports multiwindow and webview. For MVP:

- Tauri creates a main window
- Main window runs a webview showing the Mantine React UI (controls, roster, etc.)
- Bevy creates its own OS window next to / overlapping the Tauri window
- Communication: Tauri `invoke` (Rust→JS) and `emit` (JS→Rust) for UI state
- The Bevy scene and WebSocket client run in the Tauri Rust backend as a
  separate thread
- Bevy's `Window` is created with `WindowPlugin` and configured via
  `Window::new()`, separate from the Tauri window

### MVP Shortcut

Instead of full Tauri+Bevy integration (complex, platform-specific), the
MVP demo can run as:
1. **Standalone server**: `cargo run -p warren-server -- --port 3000`
2. **Standalone client**: `cargo run -p warren-game -- --server ws://localhost:3000`
3. This proves the collaboration loop without Tauri complexity
4. Tauri wraps both after the core loop works

This phased approach: prove the hard part (collaboration) first, then
wrap in Tauri for the polished desktop experience.

## 7. Task Breakdown

### Phase 1: Foundation (Foundation)
- [ ] Fix workspace: update Bevy to 0.20, add `warren-server` binary, fix `game` lib
- [ ] Define shared types in a `protocol` module (or shared crate)
- [ ] `cargo check --workspace` passes
- [ ] OKF: record MVP plan as a concept
- [ ] Create `warren-protocol` shared crate for shared types

### Phase 2: Server
- [ ] `warren-server` main: tokio server, WebSocket accept
- [ ] `VoxelWorld` struct with `HashMap<IVec3, Voxel>`
- [ ] Handle `Join`/`Leave`/`Heartbeat`/`VoxelEdit`
- [ ] `broadcast` channel fan-out
- [ ] JSON serialization of all message types
- [ ] Unit test: server accepts edit, broadcasts to 2nd client
- [ ] Integration test: 2 clients, 2nd sees 1st's edit

### Phase 3: Bevy Voxel Rendering
- [ ] `VoxelWorld` Bevy resource + component
- [ ] Insert voxel cubes on spawn/edit
- [ ] Remove voxels on erase
- [ ] Camera + navigation (orbit + WASD)
- [ ] Click raycast: detect which voxel was hit
- [ ] Brush input: place/erase based on click
- [ ] `cargo run -p warren-game` shows scene

### Phase 4: Client ↔ Server
- [ ] WebSocket client in `warren-game`
- [ ] Send `VoxelEdit` to server on local edit
- [ ] Receive `VoxelEdit` from server, apply to local scene
- [ ] Roster display in UI
- [ ] Two instances see each other's edits

### Phase 5: Mantine UI
- [ ] Tauri webview runs Mantine React app
- [ ] Color picker (Mantine `ColorPicker` or palette grid)
- [ ] Toolbar: Place mode / Erase mode
- [ ] Online users panel (avatars, names)
- [ ] Connection status indicator
- [ ] IPC: Mantine → Tauri → Bevy (send tool state)
- [ ] IPC: Bevy → Tauri → Mantine (update roster, connection status)

### Phase 6: Tauri Integration
- [ ] Tauri binary launches Bevy engine + Mantine webview
- [ ] Tauri commands to start/stop server
- [ ] Tauri window management for multi-window mode
- [ ] Cross-platform: Mac, Windows, Linux

## 8. Key Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bevy 0.20 API changes | Render code needs adaptation | Start Phase 3 with `cargo run` test, check against docs |
| Tauri + Bevy window integration is complex | Blocks desktop launch | Phase 6 is last; MVP demo runs standalone binaries |
| WebSocket performance with many voxels over JSON | Edit lag at scale | MVP: few thousand voxels acceptable. Later: binary protocol or chunk-based diff |
| Conflict resolution simplicity (last-write-wins) | Lost edits in rapid conflict | MVP: acceptable. Later: per-voxel version + CRDT |
| OKF knowledge base needs maintenance | Stakes out of sync | Update as decisions are made in each phase |

## 9. Tech Decisions Log

| Decision            | Choice                        | Rationale |
|--------------------|-------------------------------|-------------|
| Voxel rendering     | Custom `Mesh3d` cubes         | No external crate; compatible with Bevy 0.20 |
| Voxel model        | Sparse `HashMap<IVec3, Voxel>`| Simplest; fast edits; good for MVP scale |
| World bounds         | Fixed [i32;3] volume          | Bounded edit space, predictable memory |
| Server authority    | Server authoritative         | Simplest conflict resolution |
| Protocol             | JSON over WebSocket           | Human-readable; easy to debug; MVP acceptable |
| Conflict resolution | Last-write-wins per voxel    | Simple; acceptable for MVP |
| Async runtime        | tokio                       | Industry standard; async I/O |
| WebSocket lib        | tokio-tungstenite           | Production-grade; async-native |

## 10. Open Questions

- [ ] What is the maximum voxel count for MVP? (64³ = 262K is fine for sparse map)
- [ ] Should the server support multiple scenes/assets or one shared scene?
- [ ] What Mantine components should the toolbar look like? (ColorPicker, Grid, etc.)
- [ ] Should clients be able to start/stop the server, or is it always a separate binary?
- [ ] How does Bevy get the initial voxel snapshot — full load on connect, or streamed?
