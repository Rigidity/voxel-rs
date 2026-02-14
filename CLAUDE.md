# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A voxel game built with Rust and Bevy 0.18. Work in progress with no established gameplay yet.

## Build & Run

```bash
cargo run
```

No test suite or linter is configured.

## Architecture

The game uses Bevy's plugin system with five main plugins (all in `src/plugins/`):

- **RegistryPlugin** (`registry.rs`) - Central registry for blocks, materials, models, and texture arrays
- **WorldPlugin** (`world.rs`) - Chunk loading/unloading, async mesh generation, save/load to disk
- **PlayerPlugin** (`player.rs`) - First-person controller with camera
- **PhysicsPlugin** (`physics.rs`) - AABB-based collision detection
- **HudPlugin** (`hud.rs`) - Crosshair overlay via bevy_egui

### Data Model

Blocks, materials, and models are separate traits that compose together:

- **Block** (`src/block.rs`) - Defines a block type, references any number of materials, and one model
- **Material** (`src/material.rs`) - Defines textures/colors for block faces, and will have properties like hardness
- **Model** (`src/model.rs`) - Defines vertex geometry (cube, slab, etc.)

Implementations live in `src/blocks/`, `src/materials/`, `src/models/`.

### Chunk System

- Chunks are 32x32x32 blocks (`src/chunk_data.rs`) using palette compression
- Regions are 16x16 chunks saved to disk via postcard serialization (`src/region.rs`)
- Mesh generation uses face culling and runs on async tasks (`src/chunk_mesh.rs`)
- Custom Bevy material with shader storage for model vertex data (`src/chunk_material.rs`)

### World Generation

Perlin noise-based terrain with layered octaves for mountains, hills, and detail (`src/world_generator.rs`).

## Cargo Profiles

- Dev profile: `opt-level = 1` for game code, `opt-level = 3` for dependencies
- Release profile: thin LTO, single codegen-unit
- `dev` feature flag enables Bevy's dynamic linking for faster compile times
