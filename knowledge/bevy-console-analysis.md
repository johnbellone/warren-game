---
type: Architecture
title: bevy_console — Half-Life-Inspired Console for Bevy
description: "Command console for Bevy with clap-based parsing, history, tab completion, ANSI colors, and Bevy log capture. WASM compatible. Half-Life-inspired design."
tags: [analysis, bevy-console, console, bevy, clap, debugging, logging, wasm]
generated: { by: agent/cli, at: "2026-11-02T21:14:39Z" }
status: stable
---

## Overview

bevy_console is a simple Half-Life-inspired command console for Bevy games, with `clap`-based command parsing, command history, tab completion, ANSI colors, and configurable key bindings.

## Features

- **Command parsing** via `clap` — commands derive `ConsoleCommand` and use `#[derive(Parser)]`
- **Command history** — browse previously-typed commands
- **Tab completion** — `Tab` key completes commands
- **ANSI colors** — colored output
- **Customizable key bindings** — override trigger key or console toggle
- **Bevy log capture** — route Bevy `log` output to the console
- **WASM support** — disable default features; should work in browser via WebGPU

## Usage

```rust
use bevy_console::{ConsoleConfiguration, ConsolePlugin};

App::new()
    .add_plugins((DefaultPlugins, ConsolePlugin))
    .insert_resource(ConsoleConfiguration { /* overrides */ ..Default::default() })
```

Define a command:

```rust
#[derive(Parser, ConsoleCommand)]
#[command(name = "example")]
struct MyCommand {
    /// Some message
    msg: String,
}

fn my_command(mut log: ConsoleCommand<MyCommand>) {
    if let Some(Ok(cmd)) = log.take() {
        // handle command
    }
}
```

Register: `.add_console_command::<MyCommand, _>(my_command);`

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Ctrl+L | Clear history |
| Ctrl+C | Clear line |
| Tab | Line completion |

## Relevance to warren-game

Not in MVP scope. Useful for debugging tools / developer console in the editor. The `ConsoleCommand` derive + `clap` pattern is ergonomic for building command systems.

# Related Concepts
- [Bevy Logs — LogPlugin, Log Levels, and once! Spam Prevention](bevy-logs-example.md): bevy_console can capture Bevy's log! / info! / warn! output and display it in the in-game console.
