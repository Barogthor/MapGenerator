# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

```bash
# Building and running
cargo build              # Build in debug mode
cargo run                # Run the game
cargo check              # Fast compilation check

# Code quality
cargo clippy             # Run linter
cargo fmt                # Format code
cargo test               # Run tests (currently none exist)

# Release builds
cargo build --release    # Optimized build
cargo run --release      # Run optimized version
```


# Coding Standards & AI instructions

## Coding style
- Your most important job is to manage your own context. Always read any relevant files BEFORE planning changes.
- When updating documentation, keep updates concise and on point to prevent bloat.
- Write code following KISS, YAGNI, and DRY principles.
- When in doubt follow proven best practices for implementation.
- Do not commit to git without user approval.
- You are allowed to modify/optimize import without my consent.
- Apply SOLID principles where relevant. Use modern framework features rather than reinventing solutions.
- Be brutally honest about whether an idea is good or bad.
- Make side effects explicit and minimal.
- After every code update, you will check code quality by running relevant commands when finished run 'cargo fmt'.

## Naming Conventions
- Structs & Traits: PascalCase (e.g., VoicePipeline)
- Functions/Methods: snake_case (e.g., process_audio)
- Constants: UPPER_SNAKE_CASE (e.g., MAX_AUDIO_SIZE)

## Documentation Requirements
- Every module needs a docstring
- Module docstring use "//!" instead of "//"
- Surround code snippet in triple backtick 
- Every public function needs a docstring
- Use markdown style in docstring
- Include type information in docstrings

## Architecture Overview

This is a **procedural map generator** built with Rust that creates realistic terrain using computational geometry. The project is organized as a workspace with three crates:
It should inspire itself with architecture coming from game engines to optimize the differents aspect of the application (Input, rendering, Systems, ...).

### Crate Structure
- **Main crate** (`/src/`) - Application logic, OpenGL rendering, and UI integration
- **Math crate** (`/math/`) - Core algorithms for map generation and computational geometry
- **UI crate** (`/ui/`) - Input handling and windowing abstractions using winit

### Key Architectural Components

**Map Generation Pipeline (math crate):**
- `map.rs` - Core map generation with configurable distance/reshaping functions
- `delaunay.rs` - Wrapper around Spade library for Delaunay triangulation  
- `voronoi.rs` - Voronoi diagram extraction from triangulation
- `color.rs` - Biome color assignment based on elevation/moisture

**Rendering System (main crate):**
- `pipeline.rs` - Three OpenGL rendering pipelines: Region (filled), Wire (boundaries), Site (points)
- Uses Glium for OpenGL abstraction with custom shaders in `/resources/shaders/`
- `tick.rs` - Frame timing and render loop management

**Application Flow:**
1. Generate random points with jittering
2. Apply Lloyd relaxation for better distribution
3. Create Delaunay triangulation using Spade
4. Extract Voronoi regions from triangulation
5. Apply noise functions for elevation/moisture
6. Assign biomes based on elevation+moisture thresholds
7. Render in real-time with interactive camera controls

### Dependencies and Key Libraries
- **Spade** - Robust 2D Delaunay triangulation
- **nalgebra-glm** - Linear algebra and 3D math
- **bracket-noise** - Procedural noise generation
- **Glium** - Safe OpenGL wrapper
- **egui** - Immediate mode GUI for parameter controls

### Shader System
Custom GLSL shaders handle three render passes:
- `map.*` - Region filling with biome colors
- `voronoi_wire.*` - Cell boundary wireframes  
- `voronoi_site.*` - Seed point visualization

### State Management
The `State` struct in `lib.rs` manages:
- Current distance function (Euclidean, Manhattan, etc.)
- Reshaping function (Linear, Smooth, Archipelago)
- UI state and camera parameters
- Background color presets