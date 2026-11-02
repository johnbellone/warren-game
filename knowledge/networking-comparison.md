---
type: Architecture
title: Bevy Networking — Cross-Crate Comparison and warren-game Recommendation
description: "Cross-crate comparison of aeronet, bevy_replicon, lightyear, and matchbox for warren-game's collaborative voxel editor. Comparison matrix, how lightyear layers bevy_replicon on aeronet, and a justified migration recommendation against warren-game's current tokio-tungstenite background thread + Arc<Mutex<VecDeque>> inbox bridge."
tags: [analysis, comparison, aeronet, bevy-replicon, lightyear, matchbox, bevy, networking, recommendation, warren-game]
generated: { by: agent/mcp, at: "2026-11-01T21:18:00Z" }
status: stable
---

## Context: warren-game's Current Network Layer
`game/src/net.rs` is a hand-rolled `tokio-tungstenite` WebSocket layer that runs on a
**standalone background thread** and bridges into Bevy via:
- `std::sync::mpsc::Sender<String>` — Bevy → network (outgoing JSON).
- `Arc<Mutex<VecDeque<String>>>` — network → Bevy (an "inbox" drained each frame in
  `process_inbox`).
- A separate `warren-server` tokio process is authoritative; clients send
  `ClientMsg::VoxelEdit` and receive `ServerMsg::VoxelEdit` / `Welcome` / `Roster`.
- Messages are JSON strings; conflict resolution is last-write-wins per voxel.

## The Four Crates at a Glance
The four are NOT independent alternatives — they are layers of one stack, plus a P2P outlier:

```
       high-level (opinionated, game-focused)
   lightyear   (uses bevy_replicon for replication + aeronet for I/O)
        |  uses            |  uses
   bevy_replicon          aeronet   (replication core ; I/O foundation)
                          |
        IO backends: channel / websocket / webtransport / iroh / steam
        integrations: aerone_replicon, lightyear

   matchbox   (P2P WebRTC outlier; no authoritative server; no Bevy 0.20 integration)
```

## Comparison Matrix
| Dimension | **aeronet** | **bevy_replicon** | **lightyear** | **matchbox** |
|---|---|---|---|---|
| Role | I/O foundation (plumbing) | ECS replication core | Full game framework | P2P WebRTC networking |
| I/O? | **Provides it** (channel/wss/wt/iroh/steam) | **None built-in** | Via aeronet | WebRTC data channels |
| Replication? | No (non-goal) | **Yes** (core feature) | Yes (via bevy_replicon) | No |
| Topology | Dedicated server / listen / P2P | Server-authoritative | Server-auth / P2P / host-client | **P2P only** (no server role) |
| Reliability/order | Explicit channels | Per-replication-rule | Per channel (order/reliab/priority) | Reliable/ordered, reliable/unordered, **unreliable** |
| Bevy 0.20 compatible | Yes (via replicon 0.45) | **0.45 = 0.20 ✓** | **0.31 = 0.20 ✓** | ❌ max integration = Bevy 0.17 |
| WASM | Yes | Yes | Yes (WebTransport) | **Yes (primary use case)** |
| Predict/rollback | No | No (3rd-party) | **Yes (1-line)** | Via GGRS feature |
| License | per-repo | Apache/MIT | per-repo | MIT/Apache |

## Key Relationships
- **lightyear = bevy_replicon + aeronet + game-logic layer.** Adopting lightyear *includes*
  bevy_replicon and a fixed set of aeronet backends. Choosing lightyear is more than choosing
  bevy_replicon or aeronet alone.
- **aeronet is the I/O foundation** shared by both bevy_replicon (via `aeronet_replicon`) and
  lightyear. It is not a replication framework on its own.
- **bevy_replicon is the replication core** the others build on; it supplies *no* I/O.
- **matchbox** is orthogonal: the only crate without a server-authoritative model, and the only
  one targeting WASM-first P2P. Also the only one incompatible with Bevy 0.20.

## Bevy-Version Summary (the decisive compatibility fact)
| Crate | Best version for Bevy 0.20 | warren-game compatible? |
|---|---|---|
| aeronet (via bevy_replicon 0.45) | 0.45 | ✅ |
| bevy_replicon | 0.45 | ✅ |
| lightyear | 0.31 | ✅ |
| matchbox (bevy_matchbox) | — | ❌ tops at Bevy 0.17 |

## Suitability for warren-game (a collaborative *editor*, not a competitive game)
warren-game needs: a way for N editors to share one voxel world, with last-write-wins on
positions, presence/roster, and (eventually) texture/model propagation. It does NOT yet need
client-side prediction, rollback, snapshot interpolation, or bandwidth priority.

- **aeronet** — right *transport* layer; `aeronet_websocket` drops in place of tokio-tungstenite.
- **bevy_replicon** — right *core* if warren-game wants ECS-driven world replication.
- **lightyear** — overkill unless warren-game grows real-time prediction/rollback behaviour.
- **matchbox** — wrong model (P2P, no authority) **and** incompatible with Bevy 0.20.

## Recommendation
**Phase 1 (MVP, keep the editor model): adopt `aeronet` + `bevy_replicon` (0.45).**
Rationale:
- Both target **Bevy 0.20** ✓ (no version change — satisfies the warren-game constraint).
- `aeronet_websocket` replaces the hand-rolled `tokio-tungstenite` thread + the
  `Arc<Mutex<VecDeque>>` inbox, moving I/O *into* the Bevy ECS and removing the cross-thread
  string-bridge hack.
- `bevy_replicon` (via `aeronet_replicon`) unifies client world state around the ECS, replacing
  the manual `process_inbox` replay of `ServerMsg::VoxelEdit` into the world.
- The existing `warren-protocol` JSON messages can remain as the `aeronet` payload while Replicon
  governs ECS state — a low-risk migration that keeps the server-authoritative, last-write-wins
  semantics already designed in `collaboration-architecture.md`.
- **Defer** `lightyear` (its prediction/interpolation machinery is unnecessary for an editor) and
  **reject** `matchbox` for the current stack (P2P model + no Bevy 0.20 integration; revisit only
  if the product shifts to a browser P2P editor).

**Transition shape (design only — no code in this goal):**
1. Add `aeronet` + `aeronet_websocket` and `bevy_replicon` + `aeronet_replicon` deps.
2. Replace `run_network_thread` + the mpSC/`Arc<Mutex<VecDeque>>` bridge with an aeronet
   `Session` inside Bevy; deliver messages via Replicon events/systems instead of `process_inbox`.
3. Keep `warren-server` as the authoritative process (aeronet can bridge to it over WebSocket), or
   move authority into a Bevy process with `bevy_replicon` (listen-server topology).
4. Migrate `VoxelEdit`/`Welcome`/`Roster` into Replicon remote events or replicated components.
5. Leave lightyear on the backlog for a future real-time-game requirement; document the revisit
   trigger (client-predicted character movement) in the backlog.

## Risks of the Recommendation
- Adding `aeronet` + `bevy_replicon` is a **dependency change** — explicitly deferred out of scope
  for *this* analysis goal; it is the design target for a future implementation goal.
- Replicating a large sparse voxel `HashMap` needs an entity/chunk representation design
  before any full world replication (not part of this goal).
- The `warren-server` process is currently a separate tokio binary; if moving authority into Bevy,
  the listen-server topology and a WebSocket bridge must be designed.

## Cross-References
- `aeronet-analysis.md` — I/O foundation and `aeronet_websocket` mapping.
- `bevy-replicon-analysis.md` — replication core, 0.45 = Bevy 0.20, messaging backends.
- `lightyear-analysis.md` — full framework; overkill for the editor; revisit trigger.
- `matchbox-analysis.md` — P2P outlier + Bevy 0.20 incompatibility.
- `collaboration-architecture.md` — warren-game's existing last-write-wins server model.
- `components-game.md` — current `net.rs` thread+inbox design being migrated away from.
