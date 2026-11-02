# Warren Game Editor

A Tauri-based desktop application with embedded Bevy engine for live game development.

## Architecture Overview

This project is structured as a **Tauri desktop application** that embeds a **Bevy game engine frame** for live editing functionality:

1. **Frontend**: Bun web application using Mantine 9 framework
2. **Backend**: Tauri application with Rust backend 
3. **Embedded Engine**: Bevy game engine integrated for real-time editing

## Project Structure

- `src-tauri/` - Tauri application and Rust backend
- `src/web/` - Bun web frontend (Mantine 9 + extensions)
- `bevy/` - Bevy engine integration components

## Features

This editor will allow live editing of game content with real-time preview in the embedded Bevy engine.

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

## Important Notes

This is an advanced project that requires setting up the integration between:
- Tauri desktop application
- Bun web frontend using Mantine 9 
- Embedded Bevy engine for live editing functionality

The project will require substantial configuration and custom tooling to properly integrate all components.