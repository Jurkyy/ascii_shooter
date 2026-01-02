//! Enemy AI and spawning system
//! Phase 4: Enemy entities, patrol, death states

use bevy::prelude::*;

use crate::combat::{DamageEvent, Dead, DeathEvent, Health, Shootable, Weapon};
use crate::level::{BoxCollider, ARENA_SIZE};
use crate::player::Player;
use crate::rendering::AsciiPatternId;
use crate::GameState;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_enemies)
            .add_systems(
                Update,
                (
                    update_wander_targets,
                    enemy_ai_update,
                    enemy_movement,
                    enemy_collision,
                    enemy_melee_attack,
                    enemy_ranged_attack,
                    update_enemy_projectiles,
                    update_enemy_explosions,
                    trigger_hit_reactions,
                    update_hit_reactions,
                    handle_enemy_death,
                    update_kill_counter,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Enemy type determines behavior and appearance
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    Melee,   // Rushes player and attacks up close
    Ranged,  // Keeps distance and shoots projectiles
}

/// Enemy component with stats
#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub sight_range: f32,
    pub preferred_range: f32, // For ranged enemies - distance to maintain
}

impl Enemy {
    pub fn melee() -> Self {
        Self {
            speed: 4.0,
            attack_damage: 10.0,
            attack_range: 3.0,
            attack_cooldown: 0.0,
            sight_range: 50.0,
            preferred_range: 2.0,
        }
    }

    pub fn ranged() -> Self {
        Self {
            speed: 3.0,
            attack_damage: 8.0,
            attack_range: 40.0,  // Can shoot from far
            attack_cooldown: 0.0,
            sight_range: 60.0,
            preferred_range: 20.0, // Tries to stay at this distance
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Self::melee()
    }
}

/// Hit reaction - makes enemies jitter when damaged
#[derive(Component)]
pub struct HitReaction {
    pub intensity: f32,
    pub offset: Vec3,
    pub rotation_offset: f32,
}

impl Default for HitReaction {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            offset: Vec3::ZERO,
            rotation_offset: 0.0,
        }
    }
}

impl HitReaction {
    pub fn trigger(&mut self, damage: f32) {
        self.intensity = (self.intensity + damage / 20.0).min(1.0);
    }
}

/// Wander behavior for idle enemies
#[derive(Component)]
pub struct WanderBehavior {
    pub target: Option<Vec3>,
    pub home_position: Vec3,
    pub wander_radius: f32,
    pub wait_timer: f32,
}

impl WanderBehavior {
    pub fn new(home: Vec3) -> Self {
        Self {
            target: None,
            home_position: home,
            wander_radius: 15.0,
            wait_timer: 0.0,
        }
    }
}

/// AI behavior states
#[derive(Component, Default, Clone)]
pub enum EnemyState {
    #[default]
    Idle,
    Wander,
    Chase,
    Attack,
    Retreat, // For ranged enemies to maintain distance
    Dead,
}

/// Enemy projectile component
#[derive(Component)]
pub struct EnemyProjectile {
    pub damage: f32,
    pub speed: f32,
    pub direction: Vec3,
    pub lifetime: f32,
    pub explosion_radius: f32,
}

/// Enemy explosion effect
#[derive(Component)]
pub struct EnemyExplosion {
    pub radius: f32,
    pub max_radius: f32,
    pub damage: f32,
    pub lifetime: f32,
    pub has_damaged: bool,
}

/// Spawn initial enemies around the arena
fn spawn_initial_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Melee enemy material - menacing red/dark
    let melee_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        emissive: LinearRgba::rgb(0.4, 0.05, 0.05),
        perceptual_roughness: 0.6,
        ..default()
    });

    // Ranged enemy material - purple/blue
    let ranged_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.2, 0.8),
        emissive: LinearRgba::rgb(0.1, 0.05, 0.4),
        perceptual_roughness: 0.6,
        ..default()
    });

    // Eye materials
    let melee_eye_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        emissive: LinearRgba::rgb(2.0, 2.0, 0.0),
        unlit: true,
        ..default()
    });

    let ranged_eye_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 1.0, 1.0),
        emissive: LinearRgba::rgb(1.0, 2.0, 2.0),
        unlit: true,
        ..default()
    });

    // Melee enemy positions
    let melee_positions = [
        Vec3::new(-30.0, 1.0, -30.0),
        Vec3::new(30.0, 1.0, -30.0),
        Vec3::new(-30.0, 1.0, 30.0),
        Vec3::new(30.0, 1.0, 30.0),
        Vec3::new(0.0, 1.0, -50.0),
        Vec3::new(0.0, 1.0, 50.0),
    ];

    // Ranged enemy positions
    let ranged_positions = [
        Vec3::new(-50.0, 1.0, 0.0),
        Vec3::new(50.0, 1.0, 0.0),
        Vec3::new(-60.0, 1.0, -60.0),
        Vec3::new(60.0, 1.0, 60.0),
    ];

    for pos in melee_positions {
        spawn_enemy(
            &mut commands,
            &mut meshes,
            &melee_material,
            &melee_eye_material,
            pos,
            EnemyType::Melee,
        );
    }

    for pos in ranged_positions {
        spawn_enemy(
            &mut commands,
            &mut meshes,
            &ranged_material,
            &ranged_eye_material,
            pos,
            EnemyType::Ranged,
        );
    }

    // Spawn kill counter HUD
    commands.spawn((
        Text::new("KILLS: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.3, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        KillCounter { kills: 0 },
    ));
}

/// Spawn a single enemy
fn spawn_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    body_material: &Handle<StandardMaterial>,
    eye_material: &Handle<StandardMaterial>,
    position: Vec3,
    enemy_type: EnemyType,
) {
    let enemy_stats = match enemy_type {
        EnemyType::Melee => Enemy::melee(),
        EnemyType::Ranged => Enemy::ranged(),
    };

    let health = match enemy_type {
        EnemyType::Melee => Health::new(50.0),
        EnemyType::Ranged => Health::new(35.0), // Ranged are squishier
    };

    let pattern = match enemy_type {
        EnemyType::Melee => AsciiPatternId::matrix_cycle(),
        EnemyType::Ranged => AsciiPatternId::binary(),
    };

    // Main body - tall capsule shape
    let body = commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.5))),
        MeshMaterial3d(body_material.clone()),
        Transform::from_translation(position),
        enemy_type,
        enemy_stats,
        EnemyState::default(),
        health,
        Shootable,
        HitReaction::default(),
        WanderBehavior::new(position),
        pattern,
    )).id();

    // Eyes - two small glowing spheres
    let eye_offset_y = 0.6;
    let eye_offset_x = 0.2;
    let eye_offset_z = -0.4;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.08))),
        MeshMaterial3d(eye_material.clone()),
        Transform::from_xyz(eye_offset_x, eye_offset_y, eye_offset_z),
        bevy::ecs::hierarchy::ChildOf(body),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.08))),
        MeshMaterial3d(eye_material.clone()),
        Transform::from_xyz(-eye_offset_x, eye_offset_y, eye_offset_z),
        bevy::ecs::hierarchy::ChildOf(body),
    ));
}

/// Update wander targets for idle enemies
fn update_wander_targets(
    mut enemy_query: Query<(&Transform, &mut WanderBehavior, &EnemyState, &Health)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (transform, mut wander, state, health) in &mut enemy_query {
        if health.is_dead() {
            continue;
        }

        // Only update wander when idle or wandering
        if !matches!(state, EnemyState::Idle | EnemyState::Wander) {
            wander.target = None;
            wander.wait_timer = 0.0;
            continue;
        }

        // Decrement wait timer
        if wander.wait_timer > 0.0 {
            wander.wait_timer -= dt;
            continue;
        }

        // Check if we reached target or need a new one
        let needs_new_target = match wander.target {
            None => true,
            Some(target) => {
                let dist = (target - transform.translation).length();
                dist < 1.0 // Reached target
            }
        };

        if needs_new_target {
            // Wait a bit before picking new target
            wander.wait_timer = 1.0 + (elapsed * 3.7).sin().abs() * 2.0;

            // Pick random point within wander radius of home
            let angle = elapsed * 2.3 + transform.translation.x * 0.1;
            let radius = wander.wander_radius * (0.3 + (elapsed * 1.7).sin().abs() * 0.7);
            let new_target = Vec3::new(
                wander.home_position.x + angle.cos() * radius,
                1.0,
                wander.home_position.z + angle.sin() * radius,
            );

            // Clamp to arena bounds
            let clamped = Vec3::new(
                new_target.x.clamp(-ARENA_SIZE + 5.0, ARENA_SIZE - 5.0),
                1.0,
                new_target.z.clamp(-ARENA_SIZE + 5.0, ARENA_SIZE - 5.0),
            );

            wander.target = Some(clamped);
        }
    }
}

/// Update enemy AI state based on player position
fn enemy_ai_update(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &Enemy, &EnemyType, &mut EnemyState, &Health, &WanderBehavior), Without<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (transform, enemy, enemy_type, mut state, health, wander) in &mut enemy_query {
        if health.is_dead() {
            *state = EnemyState::Dead;
            continue;
        }

        let enemy_pos = transform.translation;
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        match *state {
            EnemyState::Dead => {}
            EnemyState::Idle => {
                if distance < enemy.sight_range {
                    *state = EnemyState::Chase;
                } else if wander.target.is_some() && wander.wait_timer <= 0.0 {
                    *state = EnemyState::Wander;
                }
            }
            EnemyState::Wander => {
                if distance < enemy.sight_range {
                    *state = EnemyState::Chase;
                } else if wander.target.is_none() || wander.wait_timer > 0.0 {
                    *state = EnemyState::Idle;
                }
            }
            EnemyState::Chase => {
                match enemy_type {
                    EnemyType::Melee => {
                        if distance < enemy.attack_range {
                            *state = EnemyState::Attack;
                        } else if distance > enemy.sight_range * 1.5 {
                            *state = EnemyState::Idle;
                        }
                    }
                    EnemyType::Ranged => {
                        if distance < enemy.attack_range && distance > enemy.preferred_range * 0.8 {
                            *state = EnemyState::Attack;
                        } else if distance < enemy.preferred_range * 0.6 {
                            *state = EnemyState::Retreat;
                        } else if distance > enemy.sight_range * 1.5 {
                            *state = EnemyState::Idle;
                        }
                    }
                }
            }
            EnemyState::Attack => {
                match enemy_type {
                    EnemyType::Melee => {
                        if distance > enemy.attack_range * 1.5 {
                            *state = EnemyState::Chase;
                        }
                    }
                    EnemyType::Ranged => {
                        if distance < enemy.preferred_range * 0.5 {
                            *state = EnemyState::Retreat;
                        } else if distance > enemy.attack_range {
                            *state = EnemyState::Chase;
                        }
                    }
                }
            }
            EnemyState::Retreat => {
                if distance > enemy.preferred_range {
                    *state = EnemyState::Attack;
                } else if distance > enemy.sight_range * 1.5 {
                    *state = EnemyState::Idle;
                }
            }
        }
    }
}

/// Move enemies based on their AI state
fn enemy_movement(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&mut Transform, &Enemy, &EnemyType, &EnemyState, &Health, &WanderBehavior), Without<Player>>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let dt = time.delta_secs();

    for (mut transform, enemy, _enemy_type, state, health, wander) in &mut enemy_query {
        if health.is_dead() {
            continue;
        }

        let enemy_pos = transform.translation;

        match state {
            EnemyState::Wander => {
                if let Some(target) = wander.target {
                    let to_target = target - enemy_pos;
                    let horizontal = Vec3::new(to_target.x, 0.0, to_target.z);

                    if horizontal.length() > 0.5 {
                        let direction = horizontal.normalize();
                        transform.translation += direction * enemy.speed * 0.4 * dt;

                        let look_target = Vec3::new(target.x, transform.translation.y, target.z);
                        transform.look_at(look_target, Vec3::Y);
                    }
                }
            }
            EnemyState::Chase => {
                let to_player = player_pos - enemy_pos;
                let horizontal = Vec3::new(to_player.x, 0.0, to_player.z);

                if horizontal.length() > enemy.preferred_range * 0.8 {
                    let direction = horizontal.normalize();
                    transform.translation += direction * enemy.speed * dt;
                }

                let look_target = Vec3::new(player_pos.x, transform.translation.y, player_pos.z);
                transform.look_at(look_target, Vec3::Y);
            }
            EnemyState::Attack => {
                // Move slowly toward preferred range
                let to_player = player_pos - enemy_pos;
                let horizontal = Vec3::new(to_player.x, 0.0, to_player.z);
                let dist = horizontal.length();

                if dist > enemy.preferred_range * 1.1 {
                    let direction = horizontal.normalize();
                    transform.translation += direction * enemy.speed * 0.3 * dt;
                }

                let look_target = Vec3::new(player_pos.x, transform.translation.y, player_pos.z);
                transform.look_at(look_target, Vec3::Y);
            }
            EnemyState::Retreat => {
                let to_player = player_pos - enemy_pos;
                let horizontal = Vec3::new(to_player.x, 0.0, to_player.z);

                if horizontal.length() > 0.1 {
                    let direction = -horizontal.normalize(); // Move away
                    transform.translation += direction * enemy.speed * 0.8 * dt;
                }

                let look_target = Vec3::new(player_pos.x, transform.translation.y, player_pos.z);
                transform.look_at(look_target, Vec3::Y);
            }
            _ => {}
        }

        // Keep enemy at ground level
        transform.translation.y = 1.0;
    }
}

/// Handle enemy collision with walls and obstacles
fn enemy_collision(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    collider_query: Query<(&Transform, &BoxCollider), Without<Enemy>>,
) {
    let enemy_radius = 0.6;

    for mut enemy_transform in &mut enemy_query {
        let enemy_pos = enemy_transform.translation;

        for (collider_transform, collider) in &collider_query {
            let collider_pos = collider_transform.translation;
            let half = collider.half_extents;

            // Check XZ collision
            let combined_x = half.x + enemy_radius;
            let combined_z = half.z + enemy_radius;

            let diff_x = enemy_pos.x - collider_pos.x;
            let diff_z = enemy_pos.z - collider_pos.z;

            if diff_x.abs() < combined_x && diff_z.abs() < combined_z {
                // Collision detected - push out
                let pen_x = combined_x - diff_x.abs();
                let pen_z = combined_z - diff_z.abs();

                if pen_x < pen_z {
                    if diff_x > 0.0 {
                        enemy_transform.translation.x = collider_pos.x + combined_x;
                    } else {
                        enemy_transform.translation.x = collider_pos.x - combined_x;
                    }
                } else {
                    if diff_z > 0.0 {
                        enemy_transform.translation.z = collider_pos.z + combined_z;
                    } else {
                        enemy_transform.translation.z = collider_pos.z - combined_z;
                    }
                }
            }
        }

        // Clamp to arena bounds
        let bounds = ARENA_SIZE - 1.0;
        enemy_transform.translation.x = enemy_transform.translation.x.clamp(-bounds, bounds);
        enemy_transform.translation.z = enemy_transform.translation.z.clamp(-bounds, bounds);
    }
}

/// Melee enemy attack - damages player when in range
fn enemy_melee_attack(
    player_query: Query<Entity, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Enemy, &EnemyType, &EnemyState, &Health)>,
    player_transform_query: Query<&Transform, With<Player>>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let Ok(player_transform) = player_transform_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let dt = time.delta_secs();

    for (transform, mut enemy, enemy_type, state, health) in &mut enemy_query {
        if health.is_dead() || *enemy_type != EnemyType::Melee {
            continue;
        }

        enemy.attack_cooldown = (enemy.attack_cooldown - dt).max(0.0);

        if matches!(state, EnemyState::Attack) && enemy.attack_cooldown <= 0.0 {
            let distance = (player_pos - transform.translation).length();

            if distance < enemy.attack_range {
                damage_events.write(DamageEvent {
                    target: player_entity,
                    amount: enemy.attack_damage,
                    source: None,
                });
                enemy.attack_cooldown = 1.0;
            }
        }
    }
}

/// Ranged enemy attack - shoots projectiles at player
fn enemy_ranged_attack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Enemy, &EnemyType, &EnemyState, &Health)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let dt = time.delta_secs();

    let projectile_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 1.0),
        emissive: LinearRgba::rgb(2.0, 0.5, 3.0),
        unlit: true,
        ..default()
    });

    for (transform, mut enemy, enemy_type, state, health) in &mut enemy_query {
        if health.is_dead() || *enemy_type != EnemyType::Ranged {
            continue;
        }

        enemy.attack_cooldown = (enemy.attack_cooldown - dt).max(0.0);

        if matches!(state, EnemyState::Attack) && enemy.attack_cooldown <= 0.0 {
            let distance = (player_pos - transform.translation).length();

            if distance < enemy.attack_range {
                // Shoot a projectile
                let direction = (player_pos - transform.translation).normalize();
                let spawn_pos = transform.translation + direction * 0.8 + Vec3::Y * 0.3;

                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.15))),
                    MeshMaterial3d(projectile_material.clone()),
                    Transform::from_translation(spawn_pos),
                    EnemyProjectile {
                        damage: enemy.attack_damage,
                        speed: 20.0,
                        direction,
                        lifetime: 5.0,
                        explosion_radius: 3.0,
                    },
                ));

                enemy.attack_cooldown = 1.5; // Slower fire rate than melee attack speed
            }
        }
    }
}

/// Update enemy projectiles - move them and check for collisions
fn update_enemy_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &EnemyProjectile), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    collider_query: Query<(&Transform, &BoxCollider), (Without<Player>, Without<EnemyProjectile>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (entity, mut transform, projectile) in &mut projectile_query {
        // Move projectile
        transform.translation += projectile.direction * projectile.speed * dt;

        let proj_pos = transform.translation;
        let mut should_explode = false;

        // Check collision with player
        let dist_to_player = (proj_pos - player_pos).length();
        if dist_to_player < 1.5 {
            should_explode = true;
        }

        // Check collision with walls/floors (all BoxColliders)
        for (collider_transform, collider) in &collider_query {
            let collider_pos = collider_transform.translation;
            let half = collider.half_extents;

            let diff = proj_pos - collider_pos;
            if diff.x.abs() < half.x + 0.2 && diff.y.abs() < half.y + 0.2 && diff.z.abs() < half.z + 0.2 {
                should_explode = true;
                break;
            }
        }

        // Check lifetime
        if projectile.lifetime - dt <= 0.0 {
            should_explode = true;
        }

        if should_explode {
            // Spawn explosion
            spawn_enemy_explosion(
                &mut commands,
                &mut meshes,
                &mut materials,
                proj_pos,
                projectile.damage,
                projectile.explosion_radius,
            );
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn enemy explosion effect
fn spawn_enemy_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    damage: f32,
    radius: f32,
) {
    let explosion_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.2, 1.0, 0.7),
        emissive: LinearRgba::rgb(3.0, 0.5, 4.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Explosion sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.3))),
        MeshMaterial3d(explosion_material),
        Transform::from_translation(position),
        EnemyExplosion {
            radius: 0.3,
            max_radius: radius,
            damage,
            lifetime: 0.25,
            has_damaged: false,
        },
    ));

    // Explosion light
    commands.spawn((
        PointLight {
            intensity: 100000.0,
            color: Color::srgb(0.8, 0.3, 1.0),
            range: radius * 2.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(position),
        crate::combat::MuzzleFlash { lifetime: 0.15, max_lifetime: 0.15 },
    ));
}

/// Update enemy explosions - expand and deal damage to player
fn update_enemy_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Transform, &mut EnemyExplosion)>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<EnemyExplosion>)>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (entity, mut transform, mut explosion) in &mut explosion_query {
        // Expand explosion
        let expand_rate = explosion.max_radius / 0.12;
        explosion.radius = (explosion.radius + expand_rate * dt).min(explosion.max_radius);
        transform.scale = Vec3::splat(explosion.radius * 2.0);

        // Deal damage to player once when near max size
        if !explosion.has_damaged && explosion.radius > explosion.max_radius * 0.5 {
            explosion.has_damaged = true;

            let explosion_pos = transform.translation;
            let dist = (player_pos - explosion_pos).length();
            if dist < explosion.max_radius {
                // Damage falls off with distance
                let damage_mult = 1.0 - (dist / explosion.max_radius);
                damage_events.write(DamageEvent {
                    target: player_entity,
                    amount: explosion.damage * damage_mult,
                    source: None,
                });
            }
        }

        // Fade out
        explosion.lifetime -= dt;
        if explosion.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle enemy death - despawn after delay
fn handle_enemy_death(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    mut enemy_query: Query<(&mut Transform, &mut EnemyState), With<Enemy>>,
    mut kill_counter: Query<&mut KillCounter>,
) {
    for event in death_events.read() {
        if let Ok((mut transform, mut state)) = enemy_query.get_mut(event.entity) {
            *state = EnemyState::Dead;

            transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            transform.translation.y = 0.3;

            commands.entity(event.entity).insert(DespawnTimer { remaining: 3.0 });

            if let Ok(mut counter) = kill_counter.single_mut() {
                counter.kills += 1;
            }
        }
    }
}

/// Timer for delayed despawn
#[derive(Component)]
pub struct DespawnTimer {
    pub remaining: f32,
}

/// Kill counter HUD
#[derive(Component)]
pub struct KillCounter {
    pub kills: u32,
}

/// Update kill counter display
fn update_kill_counter(
    mut query: Query<(&mut Text, &KillCounter)>,
) {
    for (mut text, counter) in &mut query {
        **text = format!("KILLS: {}", counter.kills);
    }
}

/// Trigger hit reactions when enemies take damage
fn trigger_hit_reactions(
    mut damage_events: EventReader<DamageEvent>,
    mut enemy_query: Query<&mut HitReaction, With<Enemy>>,
) {
    for event in damage_events.read() {
        if let Ok(mut hit_reaction) = enemy_query.get_mut(event.target) {
            hit_reaction.trigger(event.amount);
        }
    }
}

/// Update hit reactions - apply jitter and decay
fn update_hit_reactions(
    mut enemy_query: Query<(&mut Transform, &mut HitReaction, &Health), With<Enemy>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut hit_reaction, health) in &mut enemy_query {
        if health.is_dead() {
            continue;
        }

        if hit_reaction.intensity > 0.01 {
            let t = time.elapsed_secs() * 50.0;
            let jitter_x = (t * 1.1).sin() * (t * 2.3).cos();
            let jitter_z = (t * 1.7).cos() * (t * 1.9).sin();
            let jitter_rot = (t * 3.1).sin();

            let intensity = hit_reaction.intensity;
            hit_reaction.offset = Vec3::new(
                jitter_x * intensity * 0.03,
                0.0,
                jitter_z * intensity * 0.03,
            );
            hit_reaction.rotation_offset = jitter_rot * intensity * 0.2;

            transform.translation += hit_reaction.offset;
            transform.rotation *= Quat::from_rotation_y(hit_reaction.rotation_offset);

            hit_reaction.intensity *= (1.0 - dt * 8.0).max(0.0);
        } else {
            hit_reaction.intensity = 0.0;
            hit_reaction.offset = Vec3::ZERO;
            hit_reaction.rotation_offset = 0.0;
        }
    }
}
