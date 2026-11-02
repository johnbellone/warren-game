# Warren Game Editor

A Tauri-based desktop application with embedded Bevy engine for live game editing.

## Architecture Overview

This project is structured as a **Tauri desktop application** that embeds a **Bevy game engine frame** for live editing functionality, with a **web-compatible server component** that supports multiplayer collaborative editing:

1. **Frontend**: Bun web application using Mantine 9 framework
2. **Backend**: Tauri application with Rust backend 
3. **Embedded Engine**: Bevy game engine integrated for real-time editing
4. **Multiplayer Server**: Game server that enables simultaneous asset creation and editing

## Project Structure

- `client/` - Tauri application and Rust backend
- `editor/` - Bun web frontend (Mantine 9 + extensions)
- `game/` - Bevy engine integration components  
- `server/` - Game server with WASM support and multiplayer features
- `package.json` - Node.js/Bun dependencies

## Features

This editor will allow:
- Live editing of game content with real-time preview in the embedded Bevy engine
- Collaborative asset creation where multiple users can edit simultaneously
- Web-compatible server that can run both natively and in the browser via WebAssembly

## Multiplayer Collaboration

The system supports collaborative editing where two or more users can:
- Simultaneously create and modify game assets
- See live updates from other collaborators in real-time
- Resolve conflicts when multiple users edit the same asset
- Maintain proper access controls and user sessions

## Prerequisites

- Rust toolchain
- Node.js 18+ 
- Bun
- Tauri dependencies

## Development Setup

1. Install Rust
2. Install Bun
3. Install Tauri CLI: `npm install -g @tauri-apps/cli`
4. Run in development mode: `npm run tauri dev`

## Server Component

The server component supports both native execution and WebAssembly compilation:
- Build for native: `cargo build`
- Build for WASM: `cargo build --target wasm32-unknown-unknown`

## Important Notes

This is an advanced project that requires setting up the integration between:
- Tauri desktop application
- Bun web frontend using Mantine 9 
- Embedded Bevy engine for live editing functionality
- Web-compatible server component that can run in browser via WASM with multiplayer capabilities

The project will require substantial configuration and custom tooling to properly integrate all components as designed.