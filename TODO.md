# Implementation TODO

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

### Phase 3: ASCII Post-Process Effect
- [x] Custom post-process pass using Bevy's render graph
- [x] ASCII shader with brightness-to-character mapping
- [x] 5x7 bitmap characters encoded in shader
- [x] 4 resolution presets (Ultra, HighRes, Classic, Chunky)
- [x] Monochrome green terminal mode
- [x] 6 character pattern sets:
  - [x] Standard (density ramp)
  - [x] Blocks (checkerboards, box-drawing)
  - [x] Slashes (diagonals, X patterns)
  - [x] Binary (numbers 0-9)
  - [x] Matrix Cycle (animated cycling)
  - [x] Matrix Fall (falling rain with trails)
- [x] Per-object pattern support via render layers
- [x] Pattern camera with separate render target
- [x] Pattern mesh syncing system
- [x] Scaled character rendering (smaller at higher resolutions)
- [x] Runtime controls (F1-F4 keys)
- [x] Showcase pillars demonstrating all patterns

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
```

**Tasks**:
- [ ] Weapon component and data
- [ ] Hitscan raycast from camera
- [ ] Muzzle flash effect
- [ ] Weapon switching (if multiple weapons)

### 4.2 Enemies
**File**: `src/enemies/mod.rs`

```rust
#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
}

pub enum EnemyBehavior {
    Idle,
    Patrol { waypoints: Vec<Vec3>, current: usize },
    Chase,
    Attack,
    Dead,
}
```

**Tasks**:
- [ ] Enemy spawning
- [ ] Basic AI state machine
- [ ] Movement toward player
- [ ] Death handling

### 4.3 Player Combat
**File**: `src/combat/damage.rs`

```rust
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}
```

**Tasks**:
- [ ] Health/armor components
- [ ] Damage event system
- [ ] Screen flash on damage
- [ ] Player death handling

---

## Phase 5: Playable Prototype Polish

### 5.1 Level Design
- [ ] Design small arena with cover
- [ ] Enemy spawn points
- [ ] Pickup locations (health, ammo)

### 5.2 Game Loop
- [ ] GameOver state
- [ ] Victory state
- [ ] Win condition (all enemies dead)
- [ ] Lose condition (player health <= 0)

### 5.3 HUD (ASCII style)
```
HP: ████████░░ 80/100
AR: ██████░░░░ 60/100
AMMO: 25/100
SPEED: 12.4 m/s
```

### 5.4 Audio
- [ ] Weapon fire sound
- [ ] Jump sound
- [ ] Damage taken sound
- [ ] Enemy death sound
- [ ] Pickup sound

---

## Priority Order

1. ~~Phase 3: ASCII shader~~ **DONE**
2. **Phase 4.1**: Hitscan weapon (can shoot)
3. **Phase 4.2**: Basic enemy (something to shoot)
4. **Phase 4.3**: Player health (can die)
5. **Phase 5.2**: Win/lose conditions
6. **Phase 5.3**: Full HUD
7. **Phase 5.4**: Audio

---

## Resources

- Bevy post-process example: https://bevyengine.org/examples/shaders/post-processing/
- Quake movement explained: https://adrianb.io/2015/02/14/bunnyhop.html
- WGSL spec: https://www.w3.org/TR/WGSL/
