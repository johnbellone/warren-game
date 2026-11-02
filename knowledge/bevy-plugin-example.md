---
type: Fact
title: Bevy Plugin — Custom Plugin Registration Pattern
description: "Bevy Plugin trait: implement build(&self, &mut App) to register scoped components, resources, and systems. Configuration via struct fields. Composable via add_plugins(). The foundation of Bevy app composition."
tags: [fact, bevy, plugin, architecture, custom-plugin, plugin-registration, bevy-foundation]
generated: { by: agent/cli, at: "2026-11-02T21:14:05Z" }
status: stable
---

## Overview

This example from the Bevy documentation demonstrates how to create and register a custom plugin. Plugins are the fundamental unit of Bevy application composition: a scoped group of components, resources, and systems that together provide one piece of functionality.

## Core Pattern

### 1. Define the Plugin Struct

```rust
struct PrintMessagePlugin {
    wait_duration: Duration,
    message: String,
}

impl Plugin for PrintMessagePlugin {
    fn build(&self, app: &mut App) {
        let state = PrintMessageState {
            message: self.message.clone(),
            timer: Timer::new(self.wait_duration, TimerMode::Repeating),
        };
        app.insert_resource(state)
         .add_systems(Update, print_message_system);
    }
}
```

### 2. Register on the App

```rust
App::new()
    .add_plugins((
        DefaultPlugins,
        PrintMessagePlugin {
            wait_duration: Duration::from_secs(1),
            message: "This is an example plugin".to_string(),
        },
    ))
    .run();
```

## Key Principles

| Principle | Detail |
|-----------|--------|
| **Scoping** | One plugin = one concern. Smaller is better. |
| **Configuration via struct fields** | Plugin struct holds configuration; `build()` reads it to set up resources and systems |
| **Composability** | `add_plugins()` accepts tuples; order does not matter for independent plugins |
| **Plugin trait** | Each plugin implements `Plugin::build(&self, app: &mut App)` |
| **Resources** | Plugin-specific state is registered as `Res`/`ResMut` — the natural place for a plugin's mutable data |

## Relevance to warren-game

This is the foundational Bevy pattern. The warren-game `client/src/lib.rs` Tauri IPC wrappers are essentially plugins that bridge Tauri commands to Bevy systems. Any new system (physics integration, animation, particles) in warren-game should follow this pattern and be registered via `add_plugins()`.
