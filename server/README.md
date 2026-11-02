# Warren Game Server

The server component that supports both native execution and WebAssembly compatibility.

## Features

- **Cross-platform**: Works on native platforms and in the browser via WASM
- **Bevy Integration**: Leverages Bevy engine for game logic
- **WebSockets**: Supports real-time communication with clients
- **WASM Support**: Can be compiled to WebAssembly for browser execution

## Build Targets

### Native (default)
```bash
cargo build
```

### WASM
```bash
cargo build --target wasm32-unknown-unknown
```

## Integration Points

1. **Tauri Backend**: Communicates with this server component
2. **Bevy Engine**: Uses Bevy's game loop and components
3. **Web Frontend**: Connects via WebSocket to this server
4. **Asset Management**: Coordinates game asset loading between components

## Wasm Features

The server supports WASM compilation through:
- `wasm-bindgen` for JavaScript interop
- WebSys bindings for browser APIs
- Conditional compilation using `cfg` attributes