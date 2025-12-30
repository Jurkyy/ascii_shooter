# Implementation TODO

## Completed

### Phase 1: Project Foundation
- [x] Initialize Rust project with Bevy
- [x] Set up app structure with game states (Menu, Playing, Paused)
- [x] Create test level (floor, walls, pillars)
- [x] Basic lighting (point lights, ambient)

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

### 3.1 Render Pipeline Setup
**File**: `src/rendering/mod.rs`

1. Create a custom post-process pass using Bevy's render graph
2. Sample the rendered frame as a texture input
3. Output to a fullscreen quad

**Implementation**:
```rust
// Use bevy's post_process example as reference
// Create a PostProcessPlugin with:
// - A PostProcessPipeline resource
// - A PostProcessNode for the render graph
// - Custom bind groups for the source texture
```

### 3.2 ASCII Shader (WGSL)
**File**: `assets/shaders/ascii.wgsl`

1. Divide screen into character cells (e.g., 8x12 pixels)
2. For each cell, compute average brightness
3. Map brightness to ASCII character index
4. Sample from bitmap font texture atlas

**Character ramp** (light to dark):
```
" .,:;i1tfLCG08@"
```

**Shader pseudocode**:
```wgsl
@fragment
fn fragment(uv: vec2<f32>) -> vec4<f32> {
    let cell_size = vec2<f32>(8.0, 12.0);
    let cell = floor(uv * screen_size / cell_size);
    let cell_uv = cell * cell_size / screen_size;

    // Sample multiple points in cell for average brightness
    let color = sample_cell_average(cell_uv, cell_size);
    let brightness = dot(color.rgb, vec3(0.299, 0.587, 0.114));

    // Map to character (0-15 in ramp)
    let char_index = u32(brightness * 15.0);

    // Sample font atlas at character position
    let char_uv = get_char_uv(char_index, uv, cell_size);
    let char_color = textureSample(font_atlas, sampler, char_uv);

    // Return colored ASCII or monochrome
    return vec4(color.rgb * char_color.a, 1.0);
}
```

### 3.3 Font Atlas
**File**: `assets/fonts/ascii_font.png`

- Create a 16x1 character bitmap (or 4x4 grid)
- Each character cell same size (e.g., 8x12 pixels)
- Characters: ` .,:;i1tfLCG08@`
- White on transparent background

### 3.4 Visual Options (stretch goals)
- [ ] Toggle colored vs monochrome ASCII
- [ ] Adjustable character resolution
- [ ] Scanline effect
- [ ] CRT curvature

---

## Phase 4: Combat Prototype

### 4.1 Weapons
**File**: `src/combat/weapons.rs`

```rust
#[derive(Component)]
pub struct Weapon {
    pub damage: f32,
    pub fire_rate: f32,      // shots per second
    pub spread: f32,         // accuracy cone
    pub ammo: u32,
    pub max_ammo: u32,
}

#[derive(Component)]
pub struct Hitscan;  // vs Projectile

// Systems:
// - weapon_input: detect fire button
// - hitscan_fire: raycast from camera, apply damage
// - muzzle_flash: spawn flash sprite/light
```

**Raycast implementation**:
- Cast ray from camera position in camera forward direction
- Check intersection with enemy colliders
- Apply damage to first hit

### 4.2 Enemies
**File**: `src/enemies/mod.rs`

```rust
#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct EnemyState {
    pub current: EnemyBehavior,
}

pub enum EnemyBehavior {
    Idle,
    Patrol { waypoints: Vec<Vec3>, current: usize },
    Chase,
    Attack,
    Dead,
}

// Systems:
// - enemy_ai: state machine for behavior
// - enemy_movement: move toward player or waypoints
// - enemy_death: handle death (despawn, effects)
```

### 4.3 Player Combat
**File**: `src/combat/damage.rs`

```rust
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Armor {
    pub current: f32,
    pub max: f32,
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}

// Systems:
// - apply_damage: reduce health/armor
// - damage_feedback: screen flash, sound
// - player_death: trigger game over
```

---

## Phase 5: Playable Prototype Polish

### 5.1 Level Design
- Design small arena with cover
- Place enemy spawn points
- Add pickup locations (health, ammo)

```rust
#[derive(Component)]
pub struct Pickup {
    pub kind: PickupKind,
}

pub enum PickupKind {
    Health(f32),
    Armor(f32),
    Ammo(u32),
}
```

### 5.2 Game Loop
**File**: `src/main.rs` (extend GameState)

```rust
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,  // add
    Victory,   // add
}

// Win condition: all enemies dead
// Lose condition: player health <= 0
```

### 5.3 HUD (ASCII style)
**Extend**: `src/player/mod.rs`

Display in top-left corner:
```
HP: ████████░░ 80/100
AR: ██████░░░░ 60/100
AMMO: 25/100
SPEED: 12.4 m/s
```

### 5.4 Audio (placeholder)
**Files**: `assets/sounds/`

- `shoot.ogg` - weapon fire
- `jump.ogg` - jump sound
- `hit.ogg` - damage taken
- `enemy_death.ogg` - enemy killed
- `pickup.ogg` - item collected

```rust
// Use bevy_audio
fn play_sound(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    audio.play(asset_server.load("sounds/shoot.ogg"));
}
```

---

## Priority Order

1. **Phase 3.1-3.2**: ASCII shader (core visual identity)
2. **Phase 4.1**: Hitscan weapon (can shoot)
3. **Phase 4.2**: Basic enemy (something to shoot)
4. **Phase 4.3**: Player health (can die)
5. **Phase 5.2**: Win/lose conditions
6. **Phase 5.3**: Full HUD
7. **Phase 3.4**: Visual polish
8. **Phase 5.4**: Audio

---

## Resources

- Bevy post-process example: https://bevyengine.org/examples/shaders/post-processing/
- Quake movement explained: https://adrianb.io/2015/02/14/bunnyhop.html
- WGSL spec: https://www.w3.org/TR/WGSL/
