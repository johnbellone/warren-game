---
type: Architecture
title: matchbox — P2P WebRTC Networking for Rust WASM and Native
description: "Deep analysis of johanhelsing/matchbox: UDP-like unreliable peer-to-peer WebRTC data channels for both wasm and native, with a signaling server and a bevy_matchbox plugin. Includes ggrs/leafwing integrations and the Bevy-version incompatibility flag against warren-game's Bevy 0.20 target. Reference for warren-game networking."
tags: [analysis, matchbox, bevy, networking, webrtc, p2p, signaling, ggrs, leafwing, wasm, native]
generated: { by: agent/mcp, at: "2026-11-01T21:15:00Z" }
status: stable
---

## Overview
**matchbox** (johanhelsing/matchbox) is a suite of crates providing *painless peer-to-peer
WebRTC networking* for both **WASM** and **native** Rust, using WebRTC data channels to give
**UDP-like, unordered, unreliable** connection semantics. Its goal is low-latency
multiplayer *in a browser or natively*.

- License: **MIT / Apache-2.0** dual
- Sub-crates:
  - **`matchbox_socket`** — a socket abstraction for Wasm or Native (reliable/unordered,
    reliable/ordered, and unreliable modes; configurable retransmits). Features: `ggrs`
    (a `ggrs`-compatible socket) and `tls12` (default; for native `wss://` signaling with
    servers that lack TLS 1.3).
  - **`matchbox_signaling`** — a signaling-server *library* with examples.
  - **`matchbox_server`** — a ready-to-use full-mesh signaling server.
  - **`bevy_matchbox`** — a `matchbox_socket` integration for Bevy.
- Showcase projects: NES Bundler, Cargo Space, Extreme Bevy (2-player arcade shooter),
  collaborative blackboard (Lavagna), a Janitor's Nightmare jam game.

## How It Works
- WebRTC enables direct peer↔peer connections, but a **signaling service** is needed to
  bootstrap them (offer/answer negotiation). `matchbox_server` is that service.
- After connections establish, **data flows directly peer↔peer** — no traffic through the
  signaling server from then on.
- Peers negotiate an `RTCPeerConnection`, each with one or more data channels.
- The signaling service must be reachable by all clients over http/https (production ⇒ public
  internet).
- Client-side usage is: create a socket with a signaling URL, then `.await` the message-loop
  future (or, in Bevy, `bevy_matchbox` drives this automatically); send packets via a
  non-blocking method; hook lifecycle via socket API.
- **Relay detection**: `socket.is_relayed(peer)` returns `None` until known, then
  `Some(true)` if the peer is reached through a TURN relay.

## Reliability / Ordering Semantics
- Three channel styles:
  - **reliable + ordered**
  - **reliable + unordered**
  - **unreliable** (UDP-like, for frequent low-importance updates)
- **Configurable packet retransmits**, per channel.
- WebRTC data channels are the transport; reliability is delivered *by the transport*,
  independently of the application.

## ggrs / leafwing Integrations
- **`ggrs` feature** of `matchbox_socket` exposes a ggrs-compatible socket API. ggrs is a
  rollback-netcode protocol built on unreliable data channels.
- The `bevy_ggrs` example (Extreme Bevy) shows Bevy + GGRS, driving the message loop via
  `bevy_matchbox`; a live 2-player and 4-player web demo exist.
- **`leafwing-input-manager`** integration network player inputs (referenced via the
  "extreme-bevy" tutorial on johanhelsing.studio).

## Signaling Server Options
- **`matchbox_signaling`** — write your own signaling server from the library.
- **`matchbox_server`** — a ready-to-use full-mesh signaling server (recommended for
  quick start).
- Self-host on any http/https endpoint reachable by all clients. For production, host publicly;
  data flows peer-to-peer after the handshake, so the signaling server is only a bootstrap.

## Bevy Compatibility ⚠ FLAG FOR WARren-GAME 0.20
| bevy_matchbox | matchbox_socket | (Bevy, per public table) |
|---|---|---|
| 0.15 | 0.17 | Bevy 0.17 |
| 0.14 | 0.15 | Bevy 0.15 |
| 0.13 | 0.14 | Bevy 0.14 |
| 0.11 | 0.12 | Bevy 0.12 |
| < 0.9 | (older) | Unsupported |

**bevy_matchbox integration tops out at Bevy 0.17** in its public compatibility table.
**Bevy 0.20 is NOT listed as supported by bevy_matchbox.**

⚠ **Implication for warren-game (Bevy 0.20.1 only):** the ready-made Bevy integration
(`bevy_matchbox`) is *incompatible* with warren-game's target. The raw `matchbox_socket`
crate could potentially still be targeted to 0.20 by a third-party, but as of this analysis the
official integration does not support warren-game's Bevy version — so matchbox is *not a viable
drop-in* for the current stack without a porting effort.

## Key API Concepts
- `matchbox_socket` — create with a signaling URL; `.await` message loop; `.send()` to a peer;
  `update_peers()` / `is_relayed()` / subscribe to connection events.
- Channel reliability: `reliable_ordered`, `reliable_unordered`, `unreliable`; configurable
  retransmits.
- `bevy_matchbox` plugin drives the message loop automatically inside Bevy; `ggrs` feature
  gives a `ggrs`-compatible socket + rollback.
- `matchbox_server` / `matchbox_signaling` for the bootstrap layer.

## warren-game Migration Notes
warren-game's net.rs is a **server-authoritative WebSocket** model (a `warren-server` tokio
process is authoritative and broadcasts VoxelEdits to all clients). Matchbox is the only
crate among the four that is **peer-to-peer with no authoritative server role** — a
fundamentally different model.

### What changes if matchbox were chosen
- No `warren-server`: each client talks directly to every other client (full mesh).
- A signaling server must be stood up (`matchbox_server`), reachable by all participants
  (public internet for real play).
- Conflict resolution: last-write-wins still applies, but with no authority, clients reconcile
  their own views — simpler or harder depending on design.
- **Blocker**: `bevy_matchbox` does not support Bevy 0.20; would require a port.

### When it would be the right choice
- If warren-game's target changes to a P2P/browser-based multiplayer editor *and* a
  Bevy-0.20 build of matchbox becomes available. Not viable for the MVP as-is.
- Its WASM/WebRTC strength is unique among the four: warren-game targets Tauri native (Metal),
  where WebSocket/WebTransport (aeronet) is a better-fit transport.

## Comparison Position
matchbox is the **P2P outlier**: no authoritative server, WebRTC data channels, WASM-first.
It also has the **worst version compatibility** for warren-game (no Bevy 0.20 integration).
The other three crates (aeronet, bevy_replicon, lightyear) form a coherent stack
(aeronet → bevy_replicon → lightyear) that all target Bevy 0.20.
