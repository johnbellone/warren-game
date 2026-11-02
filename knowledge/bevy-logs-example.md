---
type: Fact
title: "Bevy Logs — LogPlugin, Log Levels, and once! Spam Prevention"
description: "Bevy logging: LogPlugin with configurable level and per-crate filter. trace/debug/info/warn/error levels. once! variants log once per call site for per-frame systems. RUST_LOG env var for runtime control via tracing-subscriber."
tags: [fact, bevy, logging, log-plugin, rust-log, trace, debug, once-macro, spam-prevention]
generated: { by: agent/cli, at: "2026-11-02T21:14:39Z" }
status: stable
---

## Overview

This example from the Bevy documentation shows how to use the log system in Bevy, including multi-level logging and the `once!` family of macros to prevent log spam in per-frame systems.

## Core API

### LogPlugin Configuration

```rust
App::new()
    .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
        // level: bevy::log::Level::TRACE,
        // filter: "wgpu=warn,bevy_ecs=info".to_string(),
        ..default()
    }))
```

- **`level`** — minimum log level to print (default: INFO)
- **`filter`** — tracing-subscriber style filter string; per-crate levels

### Log Levels (increasing importance)

| Macro | Level | Default | Spam Risk |
|-------|-------|---------|-----------|
| `trace!` | TRACE | Off | High — only in debug builds |
| `debug!` | DEBUG | Off | High |
| `info!` | INFO | On | Moderate |
| `warn!` | WARN | On | Low — use sparingly |
| `error!` | ERROR | On | Low |
| `panic!` | — | — | Kills the app |

### `once!` Family — One-Time Logs

For systems that run every frame but should only log once:

```rust
// Prevent spam in per-frame systems:
info_once!("Some info which is printed only once");
warn_once!("One-time warning");

// One-time expensive computation:
once!({
    info!("doing expensive things");
    let mut a: u64 = 0;
    for i in 0..100000000 { a += i; }
    info!("result: {}", a);
});
```

Each `*_once!` variant logs at most once per call site.

### Environment Control

Set log level via `RUST_LOG`:
```bash
RUST_LOG=trace ./my_game
RUST_LOG=info,bevy_ecs=warn ./my_game
```

The filter format follows the `tracing_subscriber::EnvFilter` documentation.

## Relevance to warren-game

The current warren-game uses `eprintln!` for logging. Migrating to Bevy's `log!` / `info!` / `trace!` macros with the `once!` variants would:
1. Allow `RUST_LOG` to control verbosity in release builds
2. Enable filtering by module (e.g., `warren_game=info,warren_server=debug`)
3. The `once!` macro is valuable for the net.rs systems that run every frame

# Related Concepts
- [Bevy Plugin — Custom Plugin Registration Pattern](bevy-plugin-example.md): Bevy's LogPlugin is itself a Plugin registered via add_plugins(). Demonstrates the Plugin registration pattern in production Bevy code.
