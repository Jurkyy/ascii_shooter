# ASCII Boomer Shooter

A first-person shooter with Quake-style movement rendered with a post-process ASCII effect, built in Rust using Bevy.

## Current Status

**Phase 2 Complete** - Core movement system working with bunny hopping.

## Features

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

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| Space | Jump (hold for auto-bhop) |
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
│   │   └── mod.rs           # (placeholder for ASCII shader)
│   ├── combat/
│   │   └── mod.rs           # (placeholder)
│   └── enemies/
│       └── mod.rs           # (placeholder)
└── assets/
    ├── fonts/
    ├── shaders/
    ├── levels/
    └── sounds/
```

## Movement Tuning

Current values in `src/player/movement.rs`:

| Parameter | Value | Description |
|-----------|-------|-------------|
| sv_maxspeed | 8.0 | Ground speed cap (m/s) |
| sv_accelerate | 10.0 | Ground acceleration |
| sv_airaccelerate | 20.0 | Air acceleration |
| sv_friction | 6.0 | Ground friction |
| sv_gravity | 20.0 | Gravity (m/s²) |
| sv_jumpspeed | 7.0 | Jump velocity (m/s) |
| Max bhop speed | 30.0 | Hard cap on horizontal speed |

## Tech Stack

- **Language**: Rust
- **Engine**: Bevy 0.15
- **Physics**: Custom Quake-style (no physics crate)
- **Rendering**: Bevy 3D + custom ASCII post-process (planned)
