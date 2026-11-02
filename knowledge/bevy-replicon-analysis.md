---
type: Architecture
title: bevy_replicon — Server-Authoritative Replication for Bevy
description: "Deep analysis of simgine/bevy_replicon: automatic ECS world replication, remote events/triggers, authorization, client-side visibility control, no builtin IO (pluggable messaging backends), reflect-based serialization, no_std support, Bevy version compatibility. Reference for warren-game replication."
tags: [analysis, bevy-replicon, replication, bevy, ecs, server-authoritative, no-std, serialization, reflect]
generated: { by: agent/mcp, at: "2026-11-01T21:10:00Z" }
status: stable
---

## Overview
**bevy_replicon** (simgine/bevy_replicon, crates.io, Apache-2.0/MIT) is *the* server-authoritative
replication crate for Bevy. It replications the ECS world between a server and one or more
clients, and is designed to "abstract game logic so that singleplayer, client, dedicated server,
and listen-server configurations run simultaneously".

It is **replication-only**: it has **no built-in I/O**. Networking is supplied by a *messaging
backend* (bevy_renet, aeronet, matchbox, etc.) that the user plugs in.

- License: **Apache-2.0 / MIT** (dual)
- `no_std` + atomic-CAS-less environments (e.g. `thumbv6m`) via `critical-section`
- High test coverage, "hackable" core

## Features (verbatim from README)
- **Automatic world replication** — declare which entities/components replicate; Replicon handles the rest.
- **Remote events and triggers** — fire events across the network to/from peers.
- **Authorization support** — restrict who can spawn/change what.
- **Client-side visibility control** — decide which entities/components each client sees.
- **ECS relationships** — replicate in sync via relationships (e.g. a child with its parent).
- **State serialization from replication rules** — the same rules that declare what replicates
  drive serialization.
- **Custom (de)serialization** even for non-serde types like `Box<dyn Reflect>`.
- **No built-in I/O** — pair with any messaging library.
- Heavily optimized; robust; `no_std`.

## Messaging Backends (how I/O is supplied)
Because there is no I/O, a backend crate wires Replicon to a real transport:
- **`bevy_replicon_renet`** → `bevy_renet`
- **`bevy_replicon_renet2`** → `renet2`
- **`bevy_replicon_quinnet`** → `bevy_quinnet`
- **`aeronet_replicon`** → **aeronet** (the IO foundation; WebSocket/WebTransport/Iroh/Steam)
- **`bevy_replicon_matchbox`** → **matchbox** (P2P WebRTC)

Interpolation/rollback integrations (built *on* Replicon): `bevy_rewind` (server-authoritative
rollback, Rocket-League style), `bevy_timewarp`, `bevy_replicon_snap` (snapshot interpolation +
prediction). These are third-party, not part of Replicon itself.

## Serialization Model
- Rule-driven: the replication rules you declare determine what is serialized.
- Works with `Box<dyn Reflect>` via the Reflect system — no need to implement `serde` everywhere.
- Custom hooks for types that do not implement `serde`.

## No_std / Embedded
- Targets `no_std` and platforms without atomic compare-and-swap (`thumbv6m`), using
  `critical-section`. Not required for a Tauri desktop app, but shows it is a portable core.

## Bevy Compatibility Table (key rows)
| bevy_replicon | bevy |
|---|---|
| **0.45** | **0.20** ← warren-game's target |
| 0.41–0.44 | 0.19 |
| 0.38–0.40 | 0.18 |
| 0.36–0.37 | 0.17 |
| 0.33–0.35 | 0.16 |

**Key fact: `bevy_replicon 0.45` targets Bevy 0.20 — fully compatible with warren-game.**

## Key API Concepts
- Declare replication rules on components (e.g. `#[derive(Reflect)]` + a `Replicated`/replicate config in the plugin).
- **Remote events / triggers**: send a typed event to a peer; the peer runs a system on receipt.
- **Visibility**: per-component/entity rules for which clients receive what.
- **Authorization**: restrict spawns/mutations to allowed clients.
- Server runs the authoritative simulation; clients receive replicated state and may predict/interpolate via an integration.
- Plugin-based setup added to the `App`.

## warren-game Migration Notes
warren-game's `net.rs` today is a *broadcast-and-apply* model: a separate `warren-server`
process is authoritative, and clients send `ClientMsg::VoxelEdit` and receive
`ServerMsg::VoxelEdit`/`Welcome`/`Roster`. This is already "server-authoritative" in spirit, but
state moves as JSON messages, not as replicated ECS entities.

Two viable migration shapes:
1. **Keep messages, add Replicon for the world** — use `aeronet_replicon` + Replicon to replicate
   the voxel world / entity state to clients, while the editor sends discrete `VoxelEdit` events.
   This unifies client world state around ECS replication and drops the manual inbox replay.
2. **Full Replicon** — treat each voxel entity as replicated; use remote events for edits;
   use visibility+authorization to gate who sees/edits what.

### Cost / risk
- Replicon has no I/O — you must also add a backend (aeronet_replicon is the recommended warren-game path).
- Requires `bevy_replicon 0.45` (matches Bevy 0.20 ✓).
- Voxel world is a huge sparse `HashMap` — replicate a *world-chunk / per-cell* entity model, not
  raw JSON; design an entity-per-voxel or chunk-representation before adopting full replication.
- `warren-protocol` JSON messages can remain as the *transport payload* for `aeronet_websocket`
  even as Replicon manages the ECS state layer.

## Comparison Position
bevy_replicon is the **replication core** that the other high-level options depend on:
- `aeronet_replicon` = aeronet I/O + Replicon.
- `lightyear` = bevy_replicon + aeronet I/O + prediction/interpolation/bandwidth management
  (a full game framework).
- `bevy_replicon_matchbox` = Replicon + matchbox P2P I/O.
So choosing bevy_replicon = choosing the ECS replication model for warren-game, with I/O chosen separately later.
