# Building an ASCII Boomer Shooter in Rust

*A journey into Bevy, Quake physics, and the art of game feel*

---

## The Idea

I wanted to build something that combined a few interests: retro FPS movement, ASCII art aesthetics, and learning Rust game development. The goal? A first-person shooter with authentic Quake-style bunny hopping, rendered with a post-process ASCII effect. Think DOOM meets a terminal.

This blog documents the process of building the foundation - getting that buttery smooth movement feeling right before worrying about ASCII shaders, combat or even gameplay for that matter.

---

## Part 1: Setting Up Bevy 0.16

Bevy is a data-driven game engine built in Rust using an Entity Component System (ECS) architecture. If you're coming from Unity or Godot, it's a different mental model - instead of inheritance hierarchies, you compose entities from components and write systems that operate on them. It was quite hard getting to grips with this in the beginning, I had never worked with something like it. However, when it clicked, it started to seem like a whole different type of approach that I had never considered before was possible.

### Project Setup

```toml
# Cargo.toml
[package]
name = "ascii_shooter"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { version = "0.16", features = ["dynamic_linking"] }

# Optimize dependencies in dev for playable framerates
[profile.dev.package."*"]
opt-level = 3
```

The `dynamic_linking` feature is crucial during development - it cuts rebuild times from 30+ seconds to under 2 seconds after the initial compile. Bevy is a big crate. Perhaps a fun step in this journey could be rewriting the needed components of Bevy in my own "engine" crate?

### App Structure

Bevy apps are built by adding plugins and systems to an `App`:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, game_logic.run_if(in_state(GameState::Playing)))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}
```

States let you control when systems run. Menu logic doesn't need to run during gameplay, and player movement shouldn't work when paused.

---

## Part 2: The Test Arena

Before implementing movement, I needed somewhere to move. A simple arena with walls and pillars:

```rust
pub const ARENA_SIZE: f32 = 100.0;

fn spawn_test_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(250.0, 250.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        LevelGeometry,
    ));

    // Walls with colliders
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ARENA_SIZE * 2.0, 8.0, 0.5))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, 4.0, -ARENA_SIZE),
        BoxCollider { half_extents: Vec3::new(ARENA_SIZE, 4.0, 0.25) },
    ));
    // ... more walls
}
```

The `BoxCollider` component stores half-extents for AABB collision. This keeps collision data with the level geometry where it belongs, not hardcoded in the player code.

---

## Part 3: Quake Movement - The Secret Sauce

This is where things get interesting. Quake's movement system is legendary because of an unintended feature: bunny hopping. By jumping repeatedly and strafing in the air, players can exceed the normal speed cap.

### Why Quake Movement Feels Different

Most modern games use instant velocity changes - press forward, instantly move at max speed. Quake uses **acceleration-based movement**:

```rust
/// Quake-style ground acceleration
pub fn accelerate(
    velocity: Vec3,
    wish_dir: Vec3,    // Direction player wants to move
    wish_speed: f32,   // Max speed in that direction
    accel: f32,        // Acceleration rate
    dt: f32,
) -> Vec3 {
    // Project current velocity onto wish direction
    let current_speed = velocity.dot(wish_dir);
    let add_speed = wish_speed - current_speed;

    if add_speed <= 0.0 {
        return velocity; // Already at or above wish speed
    }

    let accel_speed = (accel * wish_speed * dt).min(add_speed);
    velocity + wish_dir * accel_speed
}
```

The key insight: acceleration is based on **velocity in the wish direction**, not total speed. This is what enables bunny hopping.

### The Bunny Hop

When you're in the air, a different function handles movement:

```rust
pub fn air_accelerate(
    velocity: Vec3,
    wish_dir: Vec3,
    wish_speed: f32,
    accel: f32,
    dt: f32,
) -> Vec3 {
    // Cap wish_speed lower than ground for air control
    let wish_speed = wish_speed.min(4.0);

    let current_speed = velocity.dot(wish_dir);
    let add_speed = wish_speed - current_speed;

    if add_speed <= 0.0 {
        return velocity;
    }

    let accel_speed = (accel * wish_speed * dt).min(add_speed);
    let new_vel = velocity + wish_dir * accel_speed;

    // Hard cap to prevent infinite speed
    let horiz_speed = Vec2::new(new_vel.x, new_vel.z).length();
    if horiz_speed > 30.0 {
        let scale = 30.0 / horiz_speed;
        return Vec3::new(new_vel.x * scale, new_vel.y, new_vel.z * scale);
    }

    new_vel
}
```

Here's the magic: when you strafe at an angle to your velocity, `current_speed` (velocity projected onto wish_dir) is low, so `add_speed` is high, so you accelerate more. By continuously turning into your strafe, you can gain speed beyond the normal cap.

### Tuning the Values

Getting the feel right required iteration. Quake's original values (320 units/sec max speed, 800 gravity) are in "Quake units" - roughly 40x larger than meters. Scaling down:

```rust
impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            sv_maxspeed: 8.0,        // ~8 m/s running
            sv_accelerate: 10.0,     // Ground acceleration
            sv_airaccelerate: 20.0,  // Air acceleration (higher = better bhop)
            sv_friction: 6.0,        // Ground friction
            sv_gravity: 20.0,        // Slightly stronger than 9.8
            sv_jumpspeed: 7.0,       // Jump velocity
            sv_stopspeed: 2.5,       // Friction cutoff
            player_height: 1.8,
            player_radius: 0.4,
        }
    }
}
```

### Auto-Bunny Hop

Classic Quake required frame-perfect jump timing. For this project, I implemented auto-bhop - hold space to jump the instant you land. Im heavily considering to rework this, but it is quite nice for testing whilst I'm still developing:

```rust
fn player_movement(/* ... */) {
    // If grounded and holding jump, jump immediately
    if state.grounded && state.wish_jump {
        velocity.0.y = config.sv_jumpspeed;
        state.grounded = false;
    }
    // ...
}

fn player_input(/* ... */) {
    // Holding space = continuously want to jump
    state.wish_jump = keyboard.pressed(KeyCode::Space);
}
```

The key to auto-bhop here, is using `pressed()` instead of `just_pressed()` - the jump triggers the frame you land, not the frame you press.

---

## Part 4: Collision Detection

I implemented simple AABB (Axis-Aligned Bounding Box) collision with wall sliding:

```rust
fn player_collision(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    collider_query: Query<(&Transform, &BoxCollider), Without<Player>>,
    config: Res<MovementConfig>,
) {
    for (mut player_transform, mut velocity) in &mut player_query {
        let player_radius = config.player_radius;

        // Floor collision
        let feet_y = player_transform.translation.y - config.player_height / 2.0;
        if feet_y < 0.0 {
            player_transform.translation.y = config.player_height / 2.0;
            if velocity.0.y < 0.0 {
                velocity.0.y = 0.0;
            }
        }

        // Wall collisions
        for (collider_transform, collider) in &collider_query {
            let half = collider.half_extents;
            let combined_x = half.x + player_radius;
            let combined_z = half.z + player_radius;

            let diff_x = player_pos.x - collider_pos.x;
            let diff_z = player_pos.z - collider_pos.z;

            if diff_x.abs() < combined_x && diff_z.abs() < combined_z {
                // Push out on smallest penetration axis
                let pen_x = combined_x - diff_x.abs();
                let pen_z = combined_z - diff_z.abs();

                if pen_x < pen_z {
                    // Push out on X, zero X velocity
                    player_transform.translation.x = collider_pos.x
                        + combined_x * diff_x.signum();
                    velocity.0.x = 0.0;
                } else {
                    // Push out on Z, zero Z velocity
                    player_transform.translation.z = collider_pos.z
                        + combined_z * diff_z.signum();
                    velocity.0.z = 0.0;
                }
            }
        }
    }
}
```

This isn't perfect (corner cases can be jittery), but it's sufficient for testing movement.

---

## Part 5: View Sway - The Game Feel

Raw movement felt sterile. Adding view effects brings the character to life:

### View Bob

A subtle camera bob when walking:

```rust
if player_state.grounded && horiz_speed > 0.5 {
    sway.bob_time += dt * 5.0;

    let bob_x = (sway.bob_time).sin() * 0.003;
    let bob_y = (sway.bob_time * 2.0).sin().abs() * 0.004;

    sway.bob_amount = Vec3::new(bob_x, bob_y, 0.0);
}
```

The Y bob uses `.abs()` on a 2x frequency sine - this creates a "step" pattern (down-up-down-up) rather than a smooth wave.

### Landing Impact

This was tricky. The naive approach - detecting when `grounded` becomes true - doesn't work for bunny hopping because the player jumps again in the same frame they land.

The solution: detect landing from **velocity change**, not ground state:

```rust
// If we were falling and suddenly stopped, we landed
let was_falling = sway.prev_velocity_y < -2.0;
let stopped_falling = velocity.0.y > sway.prev_velocity_y + 1.0
                    || velocity.0.y >= 0.0;

if was_falling && stopped_falling {
    let base_impact = (sway.prev_velocity_y.abs() / 160.0).clamp(0.0025, 0.01);

    // Reduce impact during bhop (high horizontal speed)
    let bhop_factor = if horiz_speed > 10.0 { 0.5 } else { 1.0 };

    // Blend for smoother feel
    let target = -base_impact * bhop_factor;
    sway.landing_offset = sway.landing_offset * 0.3 + target * 0.7;
}
```

The blend prevents jarring snaps - the camera eases into the dip rather than jerking down.

### Velocity Tilt

Leaning into movement direction adds momentum feel:

```rust
let forward_speed = velocity.0.dot(forward);
let right_speed = velocity.0.dot(right);

// Subtle roll and pitch based on velocity
let target_roll = -(right_speed / 60.0).clamp(-0.03, 0.03);
let target_pitch = (forward_speed / 100.0).clamp(-0.015, 0.015);

// Smooth interpolation
sway.velocity_tilt.x += (target_roll - sway.velocity_tilt.x) * dt * 5.0;
sway.velocity_tilt.y += (target_pitch - sway.velocity_tilt.y) * dt * 5.0;
```

### Viewmodel Arms

Simple cube "arms" that react to all movement effects:

```rust
// Spawn as children of camera using ChildOf component (Bevy 0.16+)
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(0.05, 0.05, 0.15))),
    MeshMaterial3d(arm_material),
    Transform::from_xyz(0.15, -0.12, -0.25),
    ViewModel,
    ChildOf(camera),
));

// In update, apply sway offsets
let vm_offset_y = sway.landing_offset * 8.0 + sway.bob_amount.y * 2.0;
vm_transform.translation.y = -0.12 + vm_offset_y;
```

The landing impact is multiplied by 8x on the viewmodel - arms should react more dramatically than the camera for visual feedback without distoring the feeling of "balance".

---

## Part 6: System Ordering

Bevy systems run in parallel by default. For movement, order matters:

```rust
.add_systems(
    Update,
    (
        player_input,      // Read input
        player_look,       // Apply mouse look
        ground_check,      // Detect if grounded
        player_movement,   // Apply acceleration/jumping
        apply_gravity,     // Apply gravity
        player_collision,  // Resolve collisions
        apply_velocity,    // Move the player
        update_view_sway,  // Update camera effects
        update_velocity_hud,
    )
        .chain()  // Force sequential execution
        .run_if(in_state(GameState::Playing)),
)
```

The `.chain()` call ensures these run in order. Without it, you might apply velocity before collision, or check ground state before movement modifies it.

---

## Current State

The movement feels great. Bunny hopping works - you can build up to 30 m/s by strafe jumping. The view effects add weight and responsiveness.

What's missing:
- ASCII post-process shader (the whole visual identity)
- Weapons and combat
- Enemies
- Game loop (win/lose conditions)

---

## What's Next

The TODO I wrote looks like this currently:
```md
## Completed

### Phase 1: Project Foundation
- [x] Initialize Rust project with Bevy
- [x] Set up app structure with game states (Menu, Playing, Paused)
- [x] Create test level (floor, walls, pillars)
- [x] Basic lighting (point lights, ambient)
- [x] Migrated to Bevy 0.16 (ChildOf hierarchy, single()/single_mut() queries)

### Phase 2: Quake-Style Movement
- [x] Player controller with Transform, Velocity, PlayerState
- [x] Mouse look (pitch clamped, yaw unlimited)
- [x] Ground movement (acceleration-based, friction, max speed)
- [x] Air movement (reduced accel, no friction)
- [x] Jumping and bunny hopping (auto-bhop on hold)
- [x] Collision detection with wall sliding
- [x] Velocity HUD
- [x] View sway system (bob, landing impact, velocity tilt)
- [x] Viewmodel arms that react to movement
---

## Phase 3: ASCII Post-Process Effect
Some fun with WGSL.

## Phase 4: Combat Prototype
???

```

Phase 3 of my TODO is the ASCII shader - the core visual hook. The plan:

1. Render the scene to a texture
2. Divide into character cells (8x12 pixels)
3. Sample average brightness per cell
4. Map to ASCII character: ` .,:;i1tfLCG08@`
5. Output colored or monochrome ASCII

After that, hitscan weapons and basic enemies to have something to shoot at.

---

## Lessons Learned

1. **Quake movement is about acceleration, not speed caps.** The bunny hop emerges from how `air_accelerate` calculates based on velocity in the wish direction.

2. **Game feel comes from small details.** View bob, landing impact, velocity tilt - none are essential, but together they transform sterile movement into something tactile.

3. **Detect events from state changes, not states.** Landing detection based on velocity delta works; grounded state doesn't because it can flip twice in one frame.

4. **Bevy's ECS encourages clean separation.** Collision data lives with level geometry. Movement config is a resource. Systems are pure functions operating on queries.

5. **Dynamic linking saves sanity.** 2-second rebuilds vs 30+ seconds makes iteration actually enjoyable.

---

*The source code is available at [github.com/Jurkyy/ascii_shooter](https://github.com/Jurkyy/ascii_shooter)*
