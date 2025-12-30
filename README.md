# ASCII Boomer Shooter

A first-person shooter with Quake-style movement rendered with a post-process ASCII effect, built in Rust using Bevy.

## Current Status

**Phase 3 Complete** - ASCII post-processing shader working with multiple presets and runtime controls.

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
- Monochrome green terminal mode
- 4 character pattern sets (Standard, Blocks, Slashes, Binary)
- Per-object pattern infrastructure (for future expansion)
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
│   │   └── mod.rs           # ASCII post-process pipeline
│   ├── combat/
│   │   └── mod.rs           # (placeholder)
│   └── enemies/
│       └── mod.rs           # (placeholder)
└── assets/
    └── shaders/
        ├── ascii.wgsl       # Main ASCII post-process shader
        └── pattern_prepass.wgsl  # Per-object pattern shader
```

## Movement Tuning

Current values in `src/player/movement.rs`:

| Parameter | Value | Description |
|-----------|-------|-------------|
| sv_maxspeed | 8.0 | Ground speed cap (m/s) |
| sv_accelerate | 10.0 | Ground acceleration |
| sv_airaccelerate | 15.0 | Air acceleration |
| sv_friction | 6.0 | Ground friction |
| sv_gravity | 20.0 | Gravity (m/s²) |
| sv_jumpspeed | 7.0 | Jump velocity (m/s) |
| Max bhop speed | 30.0 | Hard cap on horizontal speed |

## ASCII Presets

| Preset | Cell Size | Description |
|--------|-----------|-------------|
| Ultra | 3x5 | Maximum detail, tiny characters |
| High-Res | 5x9 | Good balance of detail and readability |
| Classic | 8x14 | Traditional terminal look |
| Chunky | 12x20 | Retro, large characters |

## Tech Stack

- **Language**: Rust
- **Engine**: Bevy 0.16
- **Physics**: Custom Quake-style (no physics crate)
- **Rendering**: Bevy 3D + custom WGSL ASCII post-process shader
