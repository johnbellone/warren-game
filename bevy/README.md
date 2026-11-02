# Bevy Integration

This directory contains components for integrating the Bevy game engine with the Tauri desktop application.

## Purpose

The Bevy engine integration will allow live editing of game content within the Tauri application. This component includes:

1. Bevy engine runtime
2. Communication bridge between Tauri and Bevy
3. Live preview functionality 
4. Asset loading and management

## Structure

- `src/` - Bevy engine integration code
- `Cargo.toml` - Bevy dependencies and configuration
- `build.rs` - Build scripts for Bevy integration

## Integration Points

The integration will need to bridge:
- Tauri's Rust backend with Bevy's game loop
- Web frontend with Bevy rendering
- File system access between Tauri and Bevy components