//! Combat system - weapons, damage, health/armor
//! Phase 4: Combat Prototype

use bevy::prelude::*;

use crate::GameState;

pub mod damage;
pub mod weapons;

pub use damage::*;
pub use weapons::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_systems(Startup, spawn_damage_flash_overlay)
            .add_systems(
                Update,
                (
                    update_weapon_cooldowns,
                    handle_shooting,
                    process_damage_events,
                    trigger_damage_flash,
                    update_damage_flash,
                    update_muzzle_flash,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
