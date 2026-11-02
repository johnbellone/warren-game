---
type: Architecture
title: lightyear — Server-Authoritative Game Framework for Bevy
description: "Deep analysis of cBournhonesque/lightyear: a Bevy multiplayer framework that layers bevy_replicon (ECS replication) over aeronet (I/O), with transport-agnostic backends, channels, client-side prediction, snapshot interpolation, input delay, bandwidth management, lag compensation, and deterministic replication. Reference for warren-game networking."
tags: [analysis, lightyear, bevy, networking, server-authoritative, replication, prediction, rollback, interpolation, bandwidth, lag-compensation]
generated: { by: agent/mcp, at: "2026-11-01T21:12:00Z" }
status: stable
---

## Overview
**lightyear** (cBournhonesque/lightyear, crates.io) is a Bevy multiplayer
game *framework*. It is a composite: it **uses `bevy_replicon` for ECS/world replication**
and **uses `aeronet` as its I/O layer**. Adding it gives a full, opinionated stack.

- License: per-repo (check for distribution)
- Compatible with WASM via WebTransport
- **Current: `lightyear = "0.31"` targets Bevy 0.20, Rust ≥ 1.97.1** ✓

## Relationship to bevy_replicon + aeronet
lightyear = **bevy_replicon + aeronet + game-logic layer**. It is the top of the stack:
- **aeronet** — I/O plumbing (WebSocket/WebTransport/Iroh/Steam/UDP).
- **bevy_replicon** — ECS/world replication (replication rules, interest mgmt, pre-spawning).
- **lightyear** — adds player-input handling, prediction/rollback, snapshot interpolation,
  latency-aware bandwidth management, lag compensation, and a top-level plugin/App.

So choosing lightyear = choosing bevy_replicon + a fixed set of aeronet backends, *bundled*.

## I/O Backends (transport-agnostic, via aeronet)
- **UDP sockets** — raw UDP.
- **aeronet** — WebSocket, Steam, and WebTransport.
- **WebTransport** — WASM target path.
All I/O is delegated to `aeronet` crates (see aeronet-analysis.md).

## Serialization
- Default serializer is **postcard** (binary, no alloc, good for game messages).
- User can supply a custom serialization function.

## Messaging / Channels
- **Channels** with configurable **ordering** and **reliability** guarantees.
- Automatic **packet fragmentation** for messages larger than ~1200 bytes.
- **Bandwidth management**: cap bandwidth; messages sent in decreasing priority order
  (priority is user-defined) with a priority-accumulation scheme.

## Input Handling
- Player inputs (mouse/keyboard) are **buffered every tick** on the client; lightyear ensures
  client input for tick **N** is processed on tick **N** on the server.
- **Packet-loss protection**: each packet carries the client's inputs for the last few frames.
- Integrations: **leafwing-input-manager** (`leafwing` feature auto-networks inputs)
  and **bevy-enhanced-input**.

## Advanced Replication
- **Client-side prediction** (one-line enable) with rollback on the client for responsiveness.
- **Snapshot interpolation** (one-line enable) for smooth entity interpolation when replication is infrequent.
- **Input delay**: trade responsiveness vs. fewer miss-predictions.
- **Lag compensation** so predicted entities can interact with interpolated entities (FPS use case).
- **Deterministic replication** (lockstep or prediction/rollback) when only inputs are replicated
  and the simulation is deterministic.

## Topologies
- Client-server.
- P2P.
- Host-client (a client that also acts as the server).

## Bevy Compatibility Table (key rows)
| lightyear | bevy |
|---|---|
| **0.31** | **0.20** ← warren-game's target ✓ |
| 0.28–0.30 | 0.19 |
| 0.26–0.27 | 0.18 |
| 0.25 | 0.17 |
| 0.20–0.24 | 0.16 |

**Key fact: `lightyear 0.31` targets Bevy 0.20 — fully compatible with warren-game.**

## Key API Concepts
- Top-level `lightyear` plugin added to the `App`; workspace crates under `crates/` grouped by
  role: `io` (aeronet link/udp/websocket/webtransport/crossbeam), `connection` (netcode, steam,
  raw_connection), `core` (sync/frame-interpolation/utils), `inputs`, `replication`, `transport`
  (serialization/transport/messages), `integration` (e.g. Avian), `platform`, `deterministic`, `tools`, `tests`.
- Server authoritative: the simulation runs on the server; clients predict and interpolate via a 1-line flag.
- Player inputs buffered per tick; `leafwing`/enhanced-input integrations.
- Channels for message ordering/reliability/priority; postcard serialization.

## warren-game Migration Notes
warren-game's net.rs is a **broadcast-and-apply** collaborative-editor model, not a competitive
game: voxel edits are sent to a server and broadcast to all clients with last-write-wins conflict
resolution (see collaboration-architecture.md). lightyear provides more than this model needs.

### When lightyear is the right choice
- If warren-game grows a real-time, *client-predicted* game simulation (e.g. character
  movement, latency-sensitive interaction), lightyear's prediction/rollback/interpolation are
  valuable and available with Bevy 0.20.
- It bundles bevy_replicon + aeronet — **less wiring** than adopting bevy_replicon +
  aeronet_replicon separately.

### When lightyear is *too heavy*
- collaborative *editing* last-write-wins on a sparse voxel world does not need prediction/rollback
  or bandwidth priority. A pure editor may prefer the simpler bevy_replicon + aeronet_replicon path.
- lightyear's tick/interpolation system adds a runtime overhead the MVP does not require.

## Comparison Position
lightyear = bevy_replicon + aeronet + game logic. For an editor: **lightyear provides more than
warren-game needs unless real-time prediction is a future requirement**. If warren-game stays an
editor, bevy_replicon + aeronet_replicon (separately) gives the ECS-replication core with less
overhead.
