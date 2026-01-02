use bevy::prelude::*;

use super::damage::{DamageEvent, Health};
use crate::player::{Player, PlayerCamera};
use crate::GameState;

/// Weapon stats component
#[derive(Component)]
pub struct Weapon {
    pub damage: f32,
    pub fire_rate: f32,      // Shots per second
    pub spread: f32,         // Accuracy cone in radians
    pub ammo: u32,
    pub max_ammo: u32,
    pub range: f32,          // Max hitscan distance
    pub cooldown: f32,       // Time until next shot
}

impl Weapon {
    /// Create a basic shotgun-style weapon
    pub fn shotgun() -> Self {
        Self {
            damage: 25.0,
            fire_rate: 1.5,
            spread: 0.05,
            ammo: 50,
            max_ammo: 50,
            range: 100.0,
            cooldown: 0.0,
        }
    }

    /// Create a rapid-fire machine gun
    pub fn machinegun() -> Self {
        Self {
            damage: 15.0,
            fire_rate: 10.0,
            spread: 0.02,
            ammo: 200,
            max_ammo: 200,
            range: 150.0,
            cooldown: 0.0,
        }
    }

    pub fn can_fire(&self) -> bool {
        self.cooldown <= 0.0 && self.ammo > 0
    }

    pub fn fire(&mut self) {
        if self.can_fire() {
            self.ammo -= 1;
            self.cooldown = 1.0 / self.fire_rate;
        }
    }

    pub fn update_cooldown(&mut self, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.0);
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self::machinegun()
    }
}

/// Marker for hitscan weapons (instant hit)
#[derive(Component)]
pub struct Hitscan;

/// Marker for entities that can be hit by weapons
#[derive(Component)]
pub struct Shootable;

/// Muzzle flash visual effect
#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl MuzzleFlash {
    pub fn new() -> Self {
        Self {
            lifetime: 0.05, // 50ms flash
            max_lifetime: 0.05,
        }
    }
}

/// Marker for the muzzle flash light
#[derive(Component)]
pub struct MuzzleFlashLight;

/// Update weapon cooldowns
pub fn update_weapon_cooldowns(
    mut weapons: Query<&mut Weapon>,
    time: Res<Time>,
) {
    for mut weapon in &mut weapons {
        weapon.update_cooldown(time.delta_secs());
    }
}

/// Handle shooting input
pub fn handle_shooting(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<(Entity, &mut Weapon), With<Player>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    shootable_query: Query<(Entity, &GlobalTransform), (With<Shootable>, With<Health>)>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Only fire while holding left mouse button
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, mut weapon)) = player_query.single_mut() else {
        return;
    };

    if !weapon.can_fire() {
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.single() else {
        return;
    };

    // Fire the weapon
    weapon.fire();

    // Get ray from camera center
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Simple hitscan: check against all shootable entities
    // Using sphere intersection for simplicity (proper collision would use mesh raycasting)
    let mut closest_hit: Option<(Entity, f32)> = None;

    for (entity, transform) in &shootable_query {
        let to_target = transform.translation() - ray_origin;
        let distance_along_ray = to_target.dot(ray_direction);

        // Must be in front of us and within range
        if distance_along_ray < 0.0 || distance_along_ray > weapon.range {
            continue;
        }

        // Point on ray closest to target center
        let closest_point = ray_origin + ray_direction * distance_along_ray;
        let distance_to_center = (transform.translation() - closest_point).length();

        // Hit radius (approximate - enemies are ~1 unit wide)
        let hit_radius = 1.0;

        if distance_to_center < hit_radius {
            // Check if this is closer than previous hits
            if closest_hit.is_none() || distance_along_ray < closest_hit.unwrap().1 {
                closest_hit = Some((entity, distance_along_ray));
            }
        }
    }

    // Apply damage to closest hit
    if let Some((hit_entity, _distance)) = closest_hit {
        damage_events.write(DamageEvent {
            target: hit_entity,
            amount: weapon.damage,
            source: Some(player_entity),
        });
    }

    // Spawn muzzle flash effect
    spawn_muzzle_flash(&mut commands, &mut meshes, &mut materials, camera_transform);
}

/// Spawn muzzle flash visual
fn spawn_muzzle_flash(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_transform: &GlobalTransform,
) {
    let flash_pos = camera_transform.translation() + camera_transform.forward() * 0.5
        + camera_transform.down() * 0.1
        + camera_transform.right() * 0.15;

    // Muzzle flash sprite (small bright quad)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.5),
            emissive: LinearRgba::rgb(10.0, 8.0, 2.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(flash_pos)
            .looking_at(camera_transform.translation(), Vec3::Y),
        MuzzleFlash::new(),
    ));

    // Muzzle flash point light
    commands.spawn((
        PointLight {
            intensity: 50000.0,
            color: Color::srgb(1.0, 0.8, 0.4),
            range: 10.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(flash_pos),
        MuzzleFlash::new(),
        MuzzleFlashLight,
    ));
}

/// Update and despawn muzzle flash effects
pub fn update_muzzle_flash(
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut MuzzleFlash)>,
    time: Res<Time>,
) {
    for (entity, mut flash) in &mut flash_query {
        flash.lifetime -= time.delta_secs();
        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// HUD element showing current ammo (spawned by player module)
#[derive(Component)]
pub struct AmmoHud;
