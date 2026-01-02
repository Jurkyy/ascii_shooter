use bevy::prelude::*;

/// Health component for any entity that can take damage
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        self.current / self.max
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// Armor component - absorbs damage before health
#[derive(Component)]
pub struct Armor {
    pub current: f32,
    pub max: f32,
    pub absorption: f32, // Fraction of damage absorbed (0.0-1.0)
}

impl Armor {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            absorption: 0.66, // Absorbs 66% of damage
        }
    }

    /// Returns the amount of damage that passes through to health
    pub fn absorb(&mut self, damage: f32) -> f32 {
        if self.current <= 0.0 {
            return damage;
        }

        let absorbed = damage * self.absorption;
        let armor_damage = absorbed.min(self.current);
        self.current -= armor_damage;

        // Damage that passes through = unabsorbed + leftover absorbed
        let leftover_absorbed = absorbed - armor_damage;
        damage * (1.0 - self.absorption) + leftover_absorbed
    }
}

impl Default for Armor {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Event fired when an entity takes damage
#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}

/// Event fired when an entity dies
#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

/// Marker for entities that are dead (pending cleanup)
#[derive(Component)]
pub struct Dead;

/// Process damage events - applies damage through armor to health
pub fn process_damage_events(
    mut damage_events: EventReader<DamageEvent>,
    mut death_events: EventWriter<DeathEvent>,
    mut query: Query<(&mut Health, Option<&mut Armor>)>,
) {
    for event in damage_events.read() {
        let Ok((mut health, armor)) = query.get_mut(event.target) else {
            continue;
        };

        // Skip if already dead
        if health.is_dead() {
            continue;
        }

        // Calculate final damage after armor
        let final_damage = if let Some(mut armor) = armor {
            armor.absorb(event.amount)
        } else {
            event.amount
        };

        health.take_damage(final_damage);

        // Check for death
        if health.is_dead() {
            death_events.write(DeathEvent {
                entity: event.target,
                killer: event.source,
            });
        }
    }
}

/// Screen flash effect for damage feedback
#[derive(Component)]
pub struct DamageFlash {
    pub intensity: f32,
    pub decay_rate: f32,
}

impl Default for DamageFlash {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            decay_rate: 4.0, // Fades in 0.25 seconds
        }
    }
}

/// Marker for the damage flash overlay UI element
#[derive(Component)]
pub struct DamageFlashOverlay;

/// Spawn the damage flash overlay (fullscreen red tint)
pub fn spawn_damage_flash_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(100), // On top of everything
        DamageFlashOverlay,
    ));
}

/// Update damage flash intensity when player takes damage
pub fn trigger_damage_flash(
    mut damage_events: EventReader<DamageEvent>,
    mut flash_query: Query<&mut DamageFlash>,
    player_query: Query<Entity, With<crate::player::Player>>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for event in damage_events.read() {
        if event.target == player_entity {
            if let Ok(mut flash) = flash_query.single_mut() {
                // Scale intensity by damage amount (capped)
                let intensity_boost = (event.amount / 25.0).min(1.0);
                flash.intensity = (flash.intensity + intensity_boost).min(1.0);
            }
        }
    }
}

/// Decay damage flash over time and update overlay color
pub fn update_damage_flash(
    mut flash_query: Query<&mut DamageFlash>,
    mut overlay_query: Query<&mut BackgroundColor, With<DamageFlashOverlay>>,
    time: Res<Time>,
) {
    let Ok(mut flash) = flash_query.single_mut() else {
        return;
    };

    let Ok(mut bg_color) = overlay_query.single_mut() else {
        return;
    };

    // Decay intensity
    flash.intensity = (flash.intensity - flash.decay_rate * time.delta_secs()).max(0.0);

    // Update overlay alpha
    bg_color.0 = Color::srgba(1.0, 0.0, 0.0, flash.intensity * 0.5);
}
