//! Enemy AI and spawning system
//! Phase 4: Enemy entities, patrol, death states

use bevy::prelude::*;

use crate::combat::{DamageEvent, Dead, DeathEvent, Health, Shootable, Weapon};
use crate::level::ARENA_SIZE;
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
                    enemy_ai_update,
                    enemy_movement,
                    enemy_attack,
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

/// Enemy component with stats
#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub sight_range: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            speed: 4.0,
            attack_damage: 10.0,
            attack_range: 3.0,
            attack_cooldown: 0.0,
            sight_range: 50.0,
        }
    }
}

/// Hit reaction - makes enemies jitter when damaged
#[derive(Component)]
pub struct HitReaction {
    pub intensity: f32,      // Current jitter intensity (0-1)
    pub offset: Vec3,        // Current position offset
    pub rotation_offset: f32, // Current rotation offset
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
    /// Trigger a hit reaction
    pub fn trigger(&mut self, damage: f32) {
        // Intensity scales with damage
        self.intensity = (self.intensity + damage / 20.0).min(1.0);
    }
}

/// AI behavior states
#[derive(Component, Default)]
pub enum EnemyState {
    #[default]
    Idle,
    Patrol {
        waypoints: Vec<Vec3>,
        current: usize,
    },
    Chase,
    Attack,
    Dead,
}

/// Spawn initial enemies around the arena
fn spawn_initial_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Enemy material - menacing red/dark
    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        emissive: LinearRgba::rgb(0.4, 0.05, 0.05),
        perceptual_roughness: 0.6,
        ..default()
    });

    // Eye material - glowing
    let eye_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        emissive: LinearRgba::rgb(2.0, 2.0, 0.0),
        unlit: true,
        ..default()
    });

    // Spawn positions spread around the arena
    let spawn_positions = [
        Vec3::new(-30.0, 1.0, -30.0),
        Vec3::new(30.0, 1.0, -30.0),
        Vec3::new(-30.0, 1.0, 30.0),
        Vec3::new(30.0, 1.0, 30.0),
        Vec3::new(0.0, 1.0, -50.0),
        Vec3::new(0.0, 1.0, 50.0),
        Vec3::new(-50.0, 1.0, 0.0),
        Vec3::new(50.0, 1.0, 0.0),
        Vec3::new(-60.0, 1.0, -60.0),
        Vec3::new(60.0, 1.0, 60.0),
    ];

    for pos in spawn_positions {
        spawn_enemy(&mut commands, &mut meshes, &enemy_material, &eye_material, pos);
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
) {
    // Main body - tall capsule shape
    let body = commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.5))),
        MeshMaterial3d(body_material.clone()),
        Transform::from_translation(position),
        Enemy::default(),
        EnemyState::default(),
        Health::new(50.0),
        Shootable,
        HitReaction::default(),
        AsciiPatternId::matrix_cycle(), // Enemies use animated pattern
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

/// Update enemy AI state based on player position
fn enemy_ai_update(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &Enemy, &mut EnemyState, &Health), Without<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (transform, enemy, mut state, health) in &mut enemy_query {
        // Skip dead enemies
        if health.is_dead() {
            *state = EnemyState::Dead;
            continue;
        }

        let enemy_pos = transform.translation;
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        // State transitions
        match *state {
            EnemyState::Dead => {
                // Stay dead
            }
            EnemyState::Idle | EnemyState::Patrol { .. } => {
                // If player is within sight range, start chasing
                if distance < enemy.sight_range {
                    *state = EnemyState::Chase;
                }
            }
            EnemyState::Chase => {
                if distance < enemy.attack_range {
                    *state = EnemyState::Attack;
                } else if distance > enemy.sight_range * 1.5 {
                    // Lost sight of player
                    *state = EnemyState::Idle;
                }
            }
            EnemyState::Attack => {
                if distance > enemy.attack_range * 1.5 {
                    // Player moved away, chase again
                    *state = EnemyState::Chase;
                }
            }
        }
    }
}

/// Move enemies based on their AI state
fn enemy_movement(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&mut Transform, &Enemy, &EnemyState, &Health), Without<Player>>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let dt = time.delta_secs();

    for (mut transform, enemy, state, health) in &mut enemy_query {
        if health.is_dead() {
            continue;
        }

        match state {
            EnemyState::Chase | EnemyState::Attack => {
                // Move toward player
                let enemy_pos = transform.translation;
                let to_player = player_pos - enemy_pos;
                let horizontal_to_player = Vec3::new(to_player.x, 0.0, to_player.z);

                if horizontal_to_player.length() > 0.1 {
                    let direction = horizontal_to_player.normalize();

                    // Move toward player (slower when attacking)
                    let speed = if matches!(state, EnemyState::Attack) {
                        enemy.speed * 0.3
                    } else {
                        enemy.speed
                    };

                    // Only move if not too close
                    if horizontal_to_player.length() > enemy.attack_range * 0.8 {
                        transform.translation += direction * speed * dt;
                    }

                    // Face the player
                    let look_target = Vec3::new(player_pos.x, transform.translation.y, player_pos.z);
                    transform.look_at(look_target, Vec3::Y);
                }
            }
            EnemyState::Patrol { waypoints, current } => {
                // Move toward current waypoint
                if !waypoints.is_empty() {
                    let target = waypoints[*current];
                    let to_target = target - transform.translation;
                    let horizontal = Vec3::new(to_target.x, 0.0, to_target.z);

                    if horizontal.length() > 0.5 {
                        let direction = horizontal.normalize();
                        transform.translation += direction * enemy.speed * 0.5 * dt;
                        transform.look_at(target, Vec3::Y);
                    }
                }
            }
            _ => {}
        }

        // Keep enemy at ground level
        transform.translation.y = 1.0;
    }
}

/// Enemy attack behavior - damages player when in range
fn enemy_attack(
    player_query: Query<Entity, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Enemy, &EnemyState, &Health)>,
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

    for (transform, mut enemy, state, health) in &mut enemy_query {
        if health.is_dead() {
            continue;
        }

        // Update cooldown
        enemy.attack_cooldown = (enemy.attack_cooldown - dt).max(0.0);

        if matches!(state, EnemyState::Attack) && enemy.attack_cooldown <= 0.0 {
            let distance = (player_pos - transform.translation).length();

            if distance < enemy.attack_range {
                // Attack the player
                damage_events.write(DamageEvent {
                    target: player_entity,
                    amount: enemy.attack_damage,
                    source: None, // Could track enemy entity here
                });
                enemy.attack_cooldown = 1.0; // 1 second between attacks
            }
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
            // Set to dead state
            *state = EnemyState::Dead;

            // Fall over effect - rotate and sink
            transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            transform.translation.y = 0.3;

            // Schedule despawn after a delay
            commands.entity(event.entity).insert(DespawnTimer { remaining: 3.0 });

            // Increment kill counter
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
            // Generate random-ish jitter using time as seed
            let t = time.elapsed_secs() * 50.0;
            let jitter_x = (t * 1.1).sin() * (t * 2.3).cos();
            let jitter_z = (t * 1.7).cos() * (t * 1.9).sin();
            let jitter_rot = (t * 3.1).sin();

            // Scale by intensity (subtle effect)
            let intensity = hit_reaction.intensity;
            hit_reaction.offset = Vec3::new(
                jitter_x * intensity * 0.03,
                0.0,
                jitter_z * intensity * 0.03,
            );
            hit_reaction.rotation_offset = jitter_rot * intensity * 0.2;

            // Apply offset to transform
            transform.translation += hit_reaction.offset;
            transform.rotation *= Quat::from_rotation_y(hit_reaction.rotation_offset);

            // Decay intensity quickly
            hit_reaction.intensity *= (1.0 - dt * 8.0).max(0.0);
        } else {
            hit_reaction.intensity = 0.0;
            hit_reaction.offset = Vec3::ZERO;
            hit_reaction.rotation_offset = 0.0;
        }
    }
}
