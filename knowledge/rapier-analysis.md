---
type: Architecture
title: Rapier — Multi-Platform Physics Engine (Deno/Rust)
description: "Multi-platform Rust physics engine by Dimforge: rigid bodies, force-based joints, contact events/sensors, scene queries, SIMD+parallelism, WASM via @dimforge NPM, optional cross-platform determinism. Backing engine for Avian."
tags: [analysis, rapier, physics, bevy, deno, parry, simd, wasm, determinism, fov, rigid-bodies]
generated: { by: agent/cli, at: "2026-11-02T21:12:32Z" }
status: stable
---

## Overview

Rapier is a multi-platform, open-source physics engine written in Rust by Dimforge (also makers of Parry, the collision geometry library used by Avian). It runs cross-platform including WebGL, WASM, and native.

## Features

| Feature | Detail |
|---------|--------|
| Multi-platform physics | Games, animation, robotics; cross-platform including web |
| Joint constraints | Force-based joints: ragdolls, robots, vehicles, arbitrary multi-body systems |
| Contact events & sensors | Collision and intersection events; scene queries for ray-casting, sweep tests, intersection tests |
| Performance | Optional SIMD optimizations and parallelism |
| WASM / JavaScript | Official NPM packages under `@dimforge` namespace; runs in any modern browser |
| Determinism | Optional cross-platform determinism on all IEEE 754-2008 compliant 32/64-bit platforms |
| License | FOSS, forever free |

## Relationship to Avian

Avian (`avianphysics/avian`) is the **Bevy integration layer for Rapier**. It wraps Rapier physics and Parry collision detection in Bevy-native components and systems. Use Avian when targeting Bevy; use Rapier directly for non-Bevy projects or custom integrations.

## Relevance to warren-game

Not in MVP scope (physics is listed as a "future" feature). When warren-game adds physics support, Avian is the recommended Bevy integration. Rapier directly is the alternative if a custom ECS is used.
