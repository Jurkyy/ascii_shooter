use bevy::prelude::*;

use super::damage::{DamageEvent, Health};
use crate::player::{Player, PlayerCamera};
use crate::level::BoxCollider;
use crate::GameState;

/// Weapon types available to the player
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WeaponType {
    #[default]
    Machinegun,   // 1 - Hitscan rapid fire
    RocketLauncher, // 2 - Projectile with explosion
    Sword,        // 3 - Melee swing
}

impl WeaponType {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponType::Machinegun => "MACHINEGUN",
            WeaponType::RocketLauncher => "ROCKET",
            WeaponType::Sword => "SWORD",
        }
    }
}

/// Individual weapon stats
#[derive(Clone)]
pub struct WeaponStats {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub fire_rate: f32,
    pub ammo: u32,
    pub max_ammo: u32,
    pub range: f32,
    pub cooldown: f32,
}

impl WeaponStats {
    pub fn machinegun() -> Self {
        Self {
            weapon_type: WeaponType::Machinegun,
            damage: 15.0,
            fire_rate: 10.0,
            ammo: 200,
            max_ammo: 200,
            range: 150.0,
            cooldown: 0.0,
        }
    }

    pub fn rocket_launcher() -> Self {
        Self {
            weapon_type: WeaponType::RocketLauncher,
            damage: 50.0,
            fire_rate: 1.0,
            ammo: 20,
            max_ammo: 20,
            range: 200.0,
            cooldown: 0.0,
        }
    }

    pub fn sword() -> Self {
        Self {
            weapon_type: WeaponType::Sword,
            damage: 40.0,
            fire_rate: 2.0,
            ammo: 999, // Unlimited
            max_ammo: 999,
            range: 3.0,
            cooldown: 0.0,
        }
    }

    pub fn can_fire(&self) -> bool {
        self.cooldown <= 0.0 && self.ammo > 0
    }

    pub fn fire(&mut self) {
        if self.can_fire() {
            if self.weapon_type != WeaponType::Sword {
                self.ammo -= 1;
            }
            self.cooldown = 1.0 / self.fire_rate;
        }
    }

    pub fn update_cooldown(&mut self, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.0);
    }
}

/// Player's weapon inventory - holds all weapons
#[derive(Component)]
pub struct WeaponInventory {
    pub weapons: Vec<WeaponStats>,
    pub current_index: usize,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            weapons: vec![
                WeaponStats::machinegun(),
                WeaponStats::rocket_launcher(),
                WeaponStats::sword(),
            ],
            current_index: 0,
        }
    }
}

impl WeaponInventory {
    pub fn current(&self) -> &WeaponStats {
        &self.weapons[self.current_index]
    }

    pub fn current_mut(&mut self) -> &mut WeaponStats {
        &mut self.weapons[self.current_index]
    }

    pub fn switch_to(&mut self, index: usize) {
        if index < self.weapons.len() {
            self.current_index = index;
        }
    }
}

/// Legacy Weapon component - now wraps WeaponInventory for compatibility
#[derive(Component)]
pub struct Weapon {
    pub damage: f32,
    pub fire_rate: f32,
    pub spread: f32,
    pub ammo: u32,
    pub max_ammo: u32,
    pub range: f32,
    pub cooldown: f32,
}

impl Default for Weapon {
    fn default() -> Self {
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
}

/// Marker for entities that can be hit by weapons
#[derive(Component)]
pub struct Shootable;

/// Player projectile (rockets, etc)
#[derive(Component)]
pub struct PlayerProjectile {
    pub damage: f32,
    pub speed: f32,
    pub direction: Vec3,
    pub lifetime: f32,
    pub explosion_radius: f32,
}

/// Explosion effect
#[derive(Component)]
pub struct Explosion {
    pub radius: f32,
    pub max_radius: f32,
    pub damage: f32,
    pub lifetime: f32,
    pub has_damaged: bool,
}

/// Sword swing effect
#[derive(Component)]
pub struct SwordSwing {
    pub damage: f32,
    pub lifetime: f32,
    pub has_hit: bool,
}

/// Muzzle flash visual effect
#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl MuzzleFlash {
    pub fn new() -> Self {
        Self {
            lifetime: 0.05,
            max_lifetime: 0.05,
        }
    }
}

/// Marker for the muzzle flash light
#[derive(Component)]
pub struct MuzzleFlashLight;

/// Update weapon cooldowns
pub fn update_weapon_cooldowns(
    mut inventory_query: Query<&mut WeaponInventory>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for mut inventory in &mut inventory_query {
        for weapon in &mut inventory.weapons {
            weapon.update_cooldown(dt);
        }
    }
}

/// Handle weapon switching with number keys
pub fn handle_weapon_switch(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory_query: Query<&mut WeaponInventory, With<Player>>,
) {
    let Ok(mut inventory) = inventory_query.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Digit1) {
        inventory.switch_to(0);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        inventory.switch_to(1);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        inventory.switch_to(2);
    }
}

/// Handle shooting input based on current weapon
pub fn handle_shooting(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<(Entity, &mut WeaponInventory), With<Player>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    shootable_query: Query<(Entity, &GlobalTransform), (With<Shootable>, With<Health>)>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, mut inventory)) = player_query.single_mut() else {
        return;
    };

    if !inventory.current().can_fire() {
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.single() else {
        return;
    };

    let weapon_type = inventory.current().weapon_type;
    let damage = inventory.current().damage;
    let range = inventory.current().range;

    // Fire the weapon
    inventory.current_mut().fire();

    match weapon_type {
        WeaponType::Machinegun => {
            // Hitscan
            fire_hitscan(
                player_entity,
                camera_transform,
                &shootable_query,
                &mut damage_events,
                damage,
                range,
            );
            spawn_muzzle_flash(&mut commands, &mut meshes, &mut materials, camera_transform);
        }
        WeaponType::RocketLauncher => {
            // Spawn projectile
            spawn_rocket(
                &mut commands,
                &mut meshes,
                &mut materials,
                camera_transform,
                damage,
            );
        }
        WeaponType::Sword => {
            // Melee swing
            spawn_sword_swing(
                &mut commands,
                &mut meshes,
                &mut materials,
                camera_transform,
                damage,
            );
        }
    }
}

/// Fire hitscan weapon
fn fire_hitscan(
    player_entity: Entity,
    camera_transform: &GlobalTransform,
    shootable_query: &Query<(Entity, &GlobalTransform), (With<Shootable>, With<Health>)>,
    damage_events: &mut EventWriter<DamageEvent>,
    damage: f32,
    range: f32,
) {
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    let mut closest_hit: Option<(Entity, f32)> = None;

    for (entity, transform) in shootable_query {
        let to_target = transform.translation() - ray_origin;
        let distance_along_ray = to_target.dot(ray_direction);

        if distance_along_ray < 0.0 || distance_along_ray > range {
            continue;
        }

        let closest_point = ray_origin + ray_direction * distance_along_ray;
        let distance_to_center = (transform.translation() - closest_point).length();

        let hit_radius = 1.0;

        if distance_to_center < hit_radius {
            if closest_hit.is_none() || distance_along_ray < closest_hit.unwrap().1 {
                closest_hit = Some((entity, distance_along_ray));
            }
        }
    }

    if let Some((hit_entity, _distance)) = closest_hit {
        damage_events.write(DamageEvent {
            target: hit_entity,
            amount: damage,
            source: Some(player_entity),
        });
    }
}

/// Spawn a rocket projectile
fn spawn_rocket(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_transform: &GlobalTransform,
    damage: f32,
) {
    let direction = camera_transform.forward().as_vec3();
    let spawn_pos = camera_transform.translation() + direction * 1.0;

    let rocket_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0),
        emissive: LinearRgba::rgb(3.0, 1.5, 0.0),
        unlit: true,
        ..default()
    });

    // Rocket body
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.1, 0.3))),
        MeshMaterial3d(rocket_material),
        Transform::from_translation(spawn_pos)
            .looking_to(direction, Vec3::Y),
        PlayerProjectile {
            damage,
            speed: 40.0,
            direction,
            lifetime: 5.0,
            explosion_radius: 5.0,
        },
    ));

    // Rocket trail light
    commands.spawn((
        PointLight {
            intensity: 30000.0,
            color: Color::srgb(1.0, 0.6, 0.2),
            range: 8.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(spawn_pos),
        MuzzleFlash { lifetime: 0.1, max_lifetime: 0.1 },
    ));
}

/// Spawn sword swing effect
fn spawn_sword_swing(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_transform: &GlobalTransform,
    damage: f32,
) {
    let direction = camera_transform.forward().as_vec3();
    let spawn_pos = camera_transform.translation() + direction * 1.5 + camera_transform.right().as_vec3() * 0.3;

    let sword_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 1.0),
        emissive: LinearRgba::rgb(1.0, 1.0, 2.0),
        unlit: true,
        ..default()
    });

    // Sword slash arc
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.8, 1.5))),
        MeshMaterial3d(sword_material),
        Transform::from_translation(spawn_pos)
            .looking_to(direction, Vec3::Y)
            .with_rotation(Quat::from_rotation_z(0.3)),
        SwordSwing {
            damage,
            lifetime: 0.15,
            has_hit: false,
        },
    ));
}

/// Update player projectiles
pub fn update_player_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &mut PlayerProjectile)>,
    shootable_query: Query<(Entity, &GlobalTransform), (With<Shootable>, With<Health>)>,
    collider_query: Query<(&Transform, &BoxCollider), Without<PlayerProjectile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut projectile) in &mut projectile_query {
        // Move projectile
        transform.translation += projectile.direction * projectile.speed * dt;

        // Update lifetime
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let proj_pos = transform.translation;
        let mut should_explode = false;

        // Check collision with enemies
        for (enemy_entity, enemy_transform) in &shootable_query {
            let dist = (enemy_transform.translation() - proj_pos).length();
            if dist < 1.0 {
                should_explode = true;
                break;
            }
        }

        // Check collision with walls/floors (all BoxColliders)
        for (collider_transform, collider) in &collider_query {
            let collider_pos = collider_transform.translation;
            let half = collider.half_extents;

            let diff = proj_pos - collider_pos;
            if diff.x.abs() < half.x && diff.y.abs() < half.y && diff.z.abs() < half.z {
                should_explode = true;
                break;
            }
        }

        if should_explode {
            // Spawn explosion
            spawn_explosion(
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

/// Spawn explosion effect
fn spawn_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    damage: f32,
    radius: f32,
) {
    let explosion_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.6, 0.2, 0.8),
        emissive: LinearRgba::rgb(5.0, 2.0, 0.5),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Explosion sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(explosion_material),
        Transform::from_translation(position),
        Explosion {
            radius: 0.5,
            max_radius: radius,
            damage,
            lifetime: 0.3,
            has_damaged: false,
        },
    ));

    // Explosion light
    commands.spawn((
        PointLight {
            intensity: 200000.0,
            color: Color::srgb(1.0, 0.6, 0.2),
            range: radius * 2.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(position),
        MuzzleFlash { lifetime: 0.2, max_lifetime: 0.2 },
    ));
}

/// Update explosions - expand and deal damage
pub fn update_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Transform, &mut Explosion)>,
    shootable_query: Query<(Entity, &GlobalTransform), (With<Shootable>, With<Health>)>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut explosion) in &mut explosion_query {
        // Expand explosion
        let expand_rate = explosion.max_radius / 0.15; // Reach max in 0.15s
        explosion.radius = (explosion.radius + expand_rate * dt).min(explosion.max_radius);
        transform.scale = Vec3::splat(explosion.radius * 2.0);

        // Deal damage once when near max size
        if !explosion.has_damaged && explosion.radius > explosion.max_radius * 0.5 {
            explosion.has_damaged = true;

            let explosion_pos = transform.translation;
            for (enemy_entity, enemy_transform) in &shootable_query {
                let dist = (enemy_transform.translation() - explosion_pos).length();
                if dist < explosion.max_radius {
                    // Damage falls off with distance
                    let damage_mult = 1.0 - (dist / explosion.max_radius);
                    damage_events.write(DamageEvent {
                        target: enemy_entity,
                        amount: explosion.damage * damage_mult,
                        source: None,
                    });
                }
            }
        }

        // Fade out
        explosion.lifetime -= dt;
        if explosion.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Update sword swings
pub fn update_sword_swings(
    mut commands: Commands,
    mut swing_query: Query<(Entity, &GlobalTransform, &mut SwordSwing)>,
    shootable_query: Query<(Entity, &GlobalTransform), (With<Shootable>, With<Health>)>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, transform, mut swing) in &mut swing_query {
        // Check for hits
        if !swing.has_hit {
            let swing_pos = transform.translation();
            for (enemy_entity, enemy_transform) in &shootable_query {
                let dist = (enemy_transform.translation() - swing_pos).length();
                if dist < 2.5 {
                    damage_events.write(DamageEvent {
                        target: enemy_entity,
                        amount: swing.damage,
                        source: None,
                    });
                    swing.has_hit = true;
                    break;
                }
            }
        }

        swing.lifetime -= dt;
        if swing.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
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

/// HUD element showing current weapon name
#[derive(Component)]
pub struct WeaponHud;
