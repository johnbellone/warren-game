---
type: Architecture
title: Pumpkin Minecraft Server — Analysis for warren-game
description: "Deep analysis of Pumpkin-MC/Pumpkin: NBT, chunk format, protocol, skin handling, TOML config, WASM plugin host — as a multiplayer and data model reference for warren-game."
tags: [analysis, pumpkin, minecraft, server, rust, nbt, protocol, multiplayer]
generated: { by: agent/mcp, at: "2026-11-01T20:00:00Z" }
status: stable
---

## Overview
Pumpkin is a Minecraft Java/Bedrock server in Rust, GPL-3.0 license.
24 crates, ~0.2.1+26.3-26.60 version targeting Minecraft 26.3.
Single binary, ~100 MB memory at idle (vs 1.8 GB vanilla JVM), starts in <1 second.
Uses reqwest, tokio, rayon, wasmtime, serde, toml, tracing, flate2, lz4-java-wrc.

## Crate Structure
| Crate | Purpose | warren-game relevance |
|---|---|---|
| `pumpkin-core` | Core server logic, event system | Reference for event-driven server architecture |
| `pumpkin-protocol` | Java/Bedrock protocol codecs | Reference for protocol wire types; warren-game uses JSON/WS instead |
| `pumpkin-nbt` | NBT (Named Binary Tags) reader/writer for level.dat, region files | Reference for saving voxel world to disk as NBT files |
| `pumpkin-world` | Chunk data, block state, region loading/saving | Architecture reference for chunk/mesh management |
| `pumpkin-data` | Static data: block registry, item registry, entity types | Reference for block/item registries |
| `pumpkin-auth` | Mojang/Microsoft auth (RSA, p384, ed25519) | Not relevant for offline mode |
| `pumpkin-plugin-api` | Plugin API for other languages | Not relevant |
| `pumpkin-wasm-host` | WASM plugin runtime (wasmtime, 9 languages) | Not relevant for MVP |
| `pumpkin-codecs` | Compression codecs (GZIP, ZLZ, LZ4) | Reference for compression if needed |
| `pumpkin-scheduler` | Async task scheduling | Reference for Bevy system scheduling |
| `pumpkin-command` | Command system | Not relevant |
| `pumpkin-inventory` | Player inventories | Not relevant |
| `pumpkin-config` | TOML config | Reference for config format |

## NBT Data Model (pumpkin-nbt)
NBT (Named Binary Tags Minecraft's binary save format, not XML):
- Types: Compound, List, Int, Long, Double, Float, Byte, Short, String, ByteArray, etc.
- Used for: `level.dat` (world metadata: spawn point, time, seed), region files (`region/r.X.Z.mca`)
- For warren-game MVP: use `serde_json` for voxel data save. NBT only needed for vanilla Minecraft save file compatibility.
- **Recommendation**: `serde_json` → `serde` → JSON; add NBT support later if vanilla save compatibility is needed.

## Chunk / World Storage (pumpkin-world)
- Minecraft chunks: 16×16×384 blocks (Minecraft 1.18+; 256 height pre-1.18)
- Anvil format: `region/region/r.X.Z.mca` — 32×32 chunk array with GZIP-compressed NBT chunks
- Each chunk: block palettes + blockstate bitsets + tile entities (NBT)
- For MVP: warren-game's `HashMap<[i32;3], Voxel>` is simpler — no chunk data needed, just a save/serialize of the sparse map
- Future: if warren-game supports larger worlds, consider chunk-based storage with `flate2` compression

## Protocol Design (pumpkin-protocol)
Pumpkin implements both Java and Bedrock editions protocol:
- **Java**: TCP + VarInt length-prefixed, then compressed (zlib/zstd/lz4) at higher versions
- **Bedrock**: TCP + RAKNet (UDP-based, custom protocol)
- **Packet flow**: Handshake → Login (RSA/AES) → Status → Play

### warren-game current protocol
- JSON over WebSocket (`tokio-tungstenite`), `tokio::sync::broadcast` (capacity 4096)
- Much simpler than Minecraft's protocol
- No compression, no encryption — fine for LAN multiplayer

### Recommendation for warren-game
- Keep JSON/WS for MVP (simple, human-readable, easy to debug)
- Add binary compression later if multiplayer latency becomes an issue
- Add encryption only for public internet play

## Player Skins (pumpkin-auth / pumpkin-data)
- Skins fetched from Mojang API: `GET https://api.mojang.com/users/profiles/minecraft/<username>`
- Returns UUID + skin URL → download `textures` field from `properties.texture`
- Texture is a 64×32 PNG (head, body, arms, legs; 16×16 face)
- For warren-game: not needed until players have avatars. For MVP: solid-colour cubes, no skins.

## Config System (pumpkin-config)
- TOML files per subsystem: `server.toml`, `world.toml`, `commands.toml`, etc.
- `toml = "1.1"` with `serde` derive
- warren-game could use TOML for editor config (default brush, default color, texture pack path)
- **Recommendation**: Add `toml` crate + `serde` to warren-game workspace for config; optional for MVP

## Key Dependencies
| Crate | Version | warren-game relevance |
|---|---|---|
| `reqwest` | 0.13+ | Not needed offline; add for fetching resource packs over HTTP |
| `tokio` | 1.54 | warren-game already uses this (indirectly via tokio-tungstenite) |
| `rayon` | 1.12 | Parallel chunk meshing — useful for future large worlds |
| `flat2`/`flate2` | 1.11 | GZIP compression for NBT save; optional, skip for MVP |
| `lz4-java-wrc` | 0.20 | Minecraft-specific compression; skip for MVP |
| `wasmtime` | 50.0 | WASM plugin host — not relevant |
| `zip` | 9.2 | ZIP resource packs — optional, use folder approach first |
| `toml` | 1.1 | Config; optional, use serde_json for MVP |
| `tracing` | 0.14 | Logging; warren-game uses simple_logger already |
| `serde_json` | 1.0 | warren-game uses this for protocol — recommended for model/blockstate parsing |

## Architecture Patterns for warren-game
1. **NBT world save**: `pumpkin-nbt` crate is the reference. For MVP: use `serde_json` to serialize the `VoxelWorld` HashMap. For multi-world: add `flate2` compression and chunked storage.
2. **Block registry**: `pumpkin-data` has a `BLOCKS` table mapping numeric IDs to block states. Warrengame will need a similar registry: block type ID → (model_name, texture_atlas_index, block_state_json_path).
3. **Event system**: Pumpkin's `pumpkin-core` has an event bus. warrengame has no event system yet — consider `tokio::sync::broadcast` for future block-placed/erased events.
4. **Config**: TOML config per subsystem. For MVP: use `serde_json` + file-based config; add `toml` later.
5. **Protocol**: No need to replicate Pumpkin's complex protocol. JSON/WS is sufficient for 2-10 user LAN play.
6. **Compression**: Skip for MVP. Add `flate2` (already in warren-protocol if needed) when bandwidth matters.
7. **Skin/Avatar**: Not needed until players exist as entities. Skip for MVP.

## Relevance to warren-game Texture Loader
Pumpkin is a server-side reference. The most relevant data for a **texture loader** is:
- Block registry format: `pumpkin-data` crate maps numeric block IDs to block states
- Model/texture loading is client-side (Minecraft client), not server-side
- For a game editor: the block registry can be used to know which textures correspond to which block types
- **Recommendation**: build a `BlockRegistry` in warren-game that maps block type → (model JSON name, texture names for 6 faces)
