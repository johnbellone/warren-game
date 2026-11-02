---
type: Architecture
title: aeronet — Bevy-Native Data Transfer Primitives
description: "Deep analysis of aecsocket/aeronet: ECS-native networking with swappable IO layers (channel, WebSocket, WebTransport, Iroh, Steam), a layered transport/reliability stack, replicon/lightyear integration, and no_std support. Reference for warren-game networking migration."
tags: [analysis, aeronet, bevy, networking, ecs-native, replication, websocket, webtransport, steam, iroh, no-std]
generated: { by: agent/mcp, at: "2026-11-01T21:05:00Z" }
status: stable
---

## Overview
**aeronet** (aecsocket/aeronet, crates.io) is a set of Bevy-native networking crates
whose single design focus is *robust, non-panicking data-transfer primitives* on top of
the ECS. It is explicitly a **foundation layer**: high-level replication, rollback, and
prediction are documented **non-goals** — those must be built on top (e.g. via
`aeronet_replicon` → `bevy_replicon`, or `lightyear`, which uses aeronet as its IO layer).

- License: per-repo (check for use in distribution)
- 100% doc coverage guaranteed via `#![warn(missing_docs)]`
- Ports `no_std` and limited-atomic platforms via `critical-section`
- Targets **any Bevy 0.20-compatible version** when integrated with bevy_replicon 0.45

## Design Goals (verbatim from README)
1. **Native to Bevy ECS** — network state represented as entities; react to (dis)connects via
   observers; send/receive data by *mutating components*.
2. **Correct and non-panicking** — explicit, documented error handling; no `unwrap()`s in
   library paths (a panicking path is a documented/contract-level bug only).
3. **Support any network topology** — dedicated server/client, listen server, peer-to-peer.
4. **Swappable IO layer** — choose any underlying byte-transfer mechanism; run multiple
   simultaneously (e.g. Steam + WebTransport together).

## Layered Architecture
aeronet is split into distinct layers. Understanding these is key to the warren-game
migration: the warren-game `net.rs` today flattens all of this into one background thread.

| Layer | Crate | Responsibility | Swappable? |
|---|---|---|---|
| IO layer (abstraction) | `aeronet_io` | Defines `Session` + behaviour; core connect/disconnect logic shared by all IO impls; sets up layers above | No (core) |
| IO layer (impl) | `aeronet_channel`, `aeronet_websocket`, `aeronet_webtransport`, `aeronet_iroh`, `aeronet_steam` | Establish/maintain a connection to a peer; detect connect/disconnect; send/receive *unreliable* packets | **Yes** (user-swappable, multiple per app) |
| Transport | `aeronet_transport` | Fragmentation, reliability, ordering; split/reassemble messages; delivery acks | Technically yes, but upper layers depend on it specifically |
| Above transport | `aeronet_replicon`, `lightyear` | High-level ECS replication / server-authoritative framework | Via integration |

## IO Layer Implementations
All marked *Complete*, native (+ most also WASM):
- **`aeronet_channel`** — in-memory MPSC channels, single-process (native + WASM). `cargo run --example channel`
- **`aeronet_websocket`** — WebSockets over TCP (native + WASM). Supports an encrypted server/client with self-signed certs (WASM needs a real, CA-trusted cert for the browser). This is the direct analogue to warren-game's current `tokio-tungstenite` layer.
- **`aeronet_webtransport`** — WebTransport over QUIC (native + WASM).
- **`aeronet_iroh`** — Iroh peer-to-peer QUIC; direct + NAT hole-punching with encrypted relay fallback; no dedicated server role (each endpoint accepts and initiates).
- **`aeronet_steam`** — Steam Networking Sockets (native only; requires Steam running locally).

## Integrations
- **`aeronet_replicon`** — high-level ECS replication via `bevy_replicon` (`move_box_server` / `move_box_client` examples).
- **`lightyear`** — high-level server-authoritative networking using aeronet as the IO layer.

## Key API Surface
- **`Session`** (in `aeronet_io`) — the central type; "a good place to get started". Represents a logical connection as ECS state.
- **IO layer impls** establish/maintain the link and report (dis)connect to the session layer.
- **`aeronet_transport`** provides the reliable/ordered/fragmented message channel on top of the unreliable IO.
- Entry examples: `echo_client.rs`, `echo_server.rs` (skim these first).
- Usage pattern (quickstart): 1) add `aeronet`, 2) add chosen IO layer, 3) study echo examples, 4) use docs.rs.

## warren-game Migration Notes
Current `game/src/net.rs` is a hand-rolled `tokio-tungstenite` WebSocket on a **standalone
background thread**, bridged into Bevy via:
- `std::sync::mpsc::Sender<String>` (Bevy → net, outgoing) and
- `Arc<Mutex<VecDeque<String>>>` (net → Bevy, the "inbox" drained each frame).
Messages are JSON strings; the server is `warren-server` (a separate tokio process, not in-process).

### What aeronet maps to warren-game's needs
- warren-game already speaks **WebSocket JSON** — `aeronet_websocket` is the drop-in IO
  layer that does the same transport but *inside* the Bevy ECS, replacing the thread+inbox hack.
- The `Arc<Mutex<VecDeque>>` inbox + `process_inbox` system can be removed in favour of
  aeronet's component-mutation / observer-based message delivery.
- Topology: warren-game is a **dedicated server** model today (a warren-server process).
  aeronet supports this via a `Session`/listener. P2P (iroh) would change the model.
- Reliability/ordering: warren-game relies on WebSocket ordered delivery for VoxelEdit
  broadcasts; `aeronet_transport` gives the same with explicit reliability/ordering flags.

### Cost / risk
- aeronet is a *foundation*; warren-game would still need `bevy_replicon` (via
  `aeronet_replicon`) for entity/component replication, or keep hand-rolling VoxelEdit
  messages on top of aeronet I/O.
- Bevy version: must pair with `bevy_replicon 0.45` (Bevy 0.20) — see bevy-replicon-analysis.
- No_std is not relevant for a Tauri desktop app; skip.
- Self-signed certs do **not** work in WASM (browser); only a concern if warren-game ships
  a web build. Native Tauri (Metal) can use self-signed locally.

## Comparison Position
aeronet is the *lowest layer*: it provides the plumbing. It is not a replication framework
and not a P2P WebRTC solution (that is matchbox). Warrengame's pragmatic MVP path is likely
aeronet_websocket (to replace the thread+inbox) + bevy_replicon (for replication), or a
lightyear one-liner if a full game framework is acceptable.
