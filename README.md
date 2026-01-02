# ASCII Boomer Shooter

A first-person shooter with Quake-style movement rendered with a post-process ASCII effect, built in Rust using Bevy.

## Current Status

**Phase 3 Complete** - ASCII post-processing with animated patterns, per-object pattern support, and resolution-scaled characters.

![Early demo](assets/images/matrix.png)

## Features

### Movement
- Quake-style movement physics (acceleration-based, not instant velocity)
- Bunny hopping with speed gains up to 30 m/s
- Air strafing and turning
- Box collision with level geometry
- Velocity HUD display
- View sway effects:
  - View bob when walking
  - Landing impact (camera dip on hard landings)
  - Velocity tilt (lean into movement direction)
  - Viewmodel arms that react to all movement

### ASCII Rendering
- Real-time post-process ASCII shader
- 4 resolution presets (Ultra 3x5, High-Res 5x9, Classic 8x14, Chunky 12x20)
- **6 character pattern sets:**
  - Standard (` .:-=+*#%@` density ramp)
  - Blocks (checkerboards, box-drawing)
  - Slashes (diagonal lines, X patterns)
  - Binary (numbers 0-9)
  - Matrix Cycle (animated cycling characters)
  - Matrix Fall (true falling rain with fading trails)
- Per-object pattern assignment via render layers
- Scaled character rendering (smaller chars at higher resolutions)
- Monochrome green terminal mode
- Brightness-boosted output for visibility

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| Space | Jump (hold for auto-bhop) |
| F1 | Cycle ASCII presets |
| F2 | Toggle monochrome mode |
| F3 | Toggle per-object patterns |
| F4 | Cycle global pattern |
| Escape | Pause |
| Enter/Space | Start game (from menu) |

## Building & Running

```bash
cargo run
```

First build will take a few minutes to compile Bevy. Subsequent builds use dynamic linking for faster iteration.

## Project Structure

```
ascii_shooter/
├── src/
│   ├── main.rs              # App setup, game states
│   ├── player/
│   │   ├── mod.rs           # Player systems, camera, HUD
│   │   ├── movement.rs      # Quake physics functions
│   │   └── input.rs         # (placeholder)
│   ├── level/
│   │   └── mod.rs           # Level geometry, colliders
│   ├── rendering/
│   │   ├── mod.rs           # ASCII post-process pipeline
│   │   └── pattern_material.rs  # Per-object pattern material
│   ├── combat/
│   │   └── mod.rs           # (placeholder)
│   └── enemies/
│       └── mod.rs           # (placeholder)
└── assets/
    ├── shaders/
    │   ├── ascii.wgsl           # Main ASCII post-process shader
    │   └── pattern_material.wgsl # Per-object pattern ID shader
    └── images/                   # Screenshots
```

## Movement Tuning

Current values in `src/player/movement.rs` - I tried maintaining them here but it became a bit descyned...
Movement is not based on Quake3 anymore like it was originally, but it is a nice hybrid between CS:GO Surfing/BHopping servers and Quake3.

## ASCII Presets

| Preset | Cell Size | Character Size | Description |
|--------|-----------|----------------|-------------|
| Ultra | 3x5 | ~5x9 | Maximum detail, dense small characters |
| High-Res | 5x9 | ~6x11 | Good balance of detail and readability |
| Classic | 8x14 | 8x14 | Traditional terminal look |
| Chunky | 12x20 | 12x20 | Retro, large characters |

## Pattern Sets

| ID | Name | Description |
|----|------|-------------|
| 0 | Standard | Classic ASCII density ramp |
| 1 | Blocks | Checkerboards and box-drawing |
| 2 | Slashes | Diagonal lines and X patterns |
| 3 | Binary | Numbers 0-9 digital look |
| 4 | Matrix Cycle | Animated cycling characters |
| 5 | Matrix Fall | Falling rain with fading trails |

## Tech Stack

- **Language**: Rust
- **Engine**: Bevy 0.16
- **Physics**: Custom Quake-style (no physics crate)
- **Rendering**: Bevy 3D + custom WGSL ASCII post-process shader

## Development Blog

- [Part 1: Quake Movement](blog/part1.md)
- [Part 2: The ASCII Shader](blog/part2.md)
- [Part 3: Animated Patterns](blog/part3.md)
