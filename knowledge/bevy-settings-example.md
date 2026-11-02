---
type: Fact
title: "Bevy Settings — SettingsGroup, SaveSettingsDeferred, and Persistent Config"
description: "Bevy SettingsPlugin pattern: SettingsGroup derive merges resource sections into one config file. SaveSettingsDeferred for debounced autosave. SaveSettingsSync::IfChanged before exit. Reverse-Domain-Name namespace."
tags: [fact, bevy, settings, plugin, persistence, config, save, before-exit, autosave]
generated: { by: agent/cli, at: "2026-11-02T21:14:39Z" }
status: stable
---

## Overview

Bevy's SettingsPlugin provides a declarative pattern for persistent application settings stored as Bevy Resources. Settings are grouped into `SettingsGroup` sections that merge in the config file.

## Architecture

### Core Trait: `SettingsGroup`

Derive `SettingsGroup` on any Bevy `Resource` to make it a settings section:

```rust
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct MySettings {
    count: i32,
}
```

### Group Name Merging

Multiple `SettingsGroup` structs with the same `group = "name"` attribute merge into one config section:

```rust
#[settings_group(group = "counter")]
struct OtherSettings { enabled: bool }  // merges with Counter section
```

### Save Strategies

| Strategy | Mechanism | Use Case |
|----------|-----------|----------|
| `SaveSettingsDeferred(Duration)` | Coalesced save: queues a save after Duration has passed since last change | Periodic autosave (e.g., 0.1s debounce) |
| `SaveSettingsSync` | Immediate synchronous save | Before app exit or critical state |
| `IfChanged` | Save only if any value changed | Avoid unnecessary disk writes |

### App Lifecycle

```rust
// Intercept window close to force save before exit:
WindowPlugin {
    exit_condition: ExitCondition::DontExit,
    ..default()
}

// In on_window_close system:
commands.queue(SaveSettingsSync::IfChanged);
commands.write_message(AppExit::Success);
```

## Configuration File

Settings are written to a platform-appropriate config location. The `SettingsPlugin::new("org.bevy.examples.settings")` constructor takes a namespace/reverse-Domain-Name.

## Relevance to warren-game

MVP scope does not mention settings. Highly relevant for:
- Persisting user preferences (camera position, brush color, selected tool)
- Saving session metadata
- Configuration for the Tauri desktop app (window size, theme, server URL)
The `SaveSettingsDeferred` debounce pattern is exactly what a real-time editor needs to avoid disk-write spam.

# Related Concepts
- [Bevy Plugin — Custom Plugin Registration Pattern](bevy-plugin-example.md): SettingsPlugin is a Plugin that manages persistent Resources via SettingsGroup. Shows the Plugin trait pattern with config via constructor args.
