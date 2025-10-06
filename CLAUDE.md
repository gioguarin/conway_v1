# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A terminal-based implementation of Conway's Game of Life written in Rust using the `ratatui` TUI framework. The application runs at 60 FPS with configurable simulation tick rates and supports interactive cell placement and pattern spawning.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run with random pattern spawning
cargo run -- --random
cargo run -- -r

# Show help
cargo run -- --help
```

## Architecture

### Core Components

**State (`State` struct)** - Main application state containing:
- `Grid`: 2D grid of cells stored as a flat `Vec<bool>` with wrapping indexing via `Index`/`IndexMut` traits
- `Cursor`: Tracks user cursor position for interactive cell placement
- `TickRate`: Controls simulation speed (Slow=1s, Normal=200ms, Fast=100ms)
- `paused`: Boolean controlling whether simulation is running

**Game Loop** (`State::run`)
- Fixed 60 FPS rendering loop using frame accumulator pattern
- Delta time-based tick accumulation for consistent simulation speed regardless of frame rate
- Event handling via polling (non-blocking) to maintain smooth rendering

**Grid Representation** (`Grid` struct)
- Flat `Vec<bool>` with custom 2D indexing: `data[row * cols + col]`
- Supports dynamic resizing on terminal resize events while preserving existing cells
- Wrapping boundaries implemented using `rem_euclid` for toroidal topology

**Patterns** (`Pattern` enum)
- Predefined Conway's Life patterns: Glider, Blinker, Toad, Beacon, Pulsar, Lightweight Spaceship
- Each pattern defined as `Vec<(usize, usize)>` of relative cell coordinates
- Random spawning mode (`--random` flag) spawns patterns at 10% probability per tick

### Game of Life Rules (implemented in `State::update`)

Standard Conway's Life rules applied each tick:
- Live cell with 2-3 neighbors survives
- Dead cell with exactly 3 neighbors becomes alive
- All other cells die or remain dead

### Rendering (`Widget for &State`)

Uses ratatui's Widget trait to render:
- Live cells: white background space
- Dead cells: empty space
- Cursor position: asterisk (`*`) with inverted colors when over live cell

## Key Implementation Details

- **Event handling**: Arrow keys move cursor (with wrapping), spacebar toggles cell, `p` pauses, `[`/`]` adjust speed
- **Terminal resize**: Grid dynamically resizes, preserving existing cells that fit in new dimensions
- **Frame rate**: Locked to 60 FPS via `sleep()` to prevent excessive CPU usage
- **Tick rate**: Simulation updates decoupled from frame rate using accumulator pattern
