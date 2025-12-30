use bevy::prelude::*;

/// Quake movement constants - tune these for feel
#[derive(Resource)]
pub struct MovementConfig {
    pub sv_maxspeed: f32,       // Max ground speed (units/sec)
    pub sv_accelerate: f32,     // Ground acceleration
    pub sv_airaccelerate: f32,  // Air acceleration
    pub sv_friction: f32,       // Ground friction
    pub sv_gravity: f32,        // Gravity (units/sec^2)
    pub sv_jumpspeed: f32,      // Jump velocity
    pub sv_stopspeed: f32,      // Speed below which friction stops you instantly
    pub player_height: f32,     // Player capsule height
    pub player_radius: f32,     // Player capsule radius
}

impl Default for MovementConfig {
    fn default() -> Self {
        // Quake values scaled down ~40x for meter-scale (1 unit = 1 meter)
        Self {
            sv_maxspeed: 8.0,        // ~8 m/s running speed
            sv_accelerate: 10.0,     // Acceleration feels good as-is
            sv_airaccelerate: 20.0,  // Air accel for bunny hop
            sv_friction: 6.0,        // Friction coefficient
            sv_gravity: 20.0,        // Slightly stronger than real (9.8)
            sv_jumpspeed: 7.0,       // ~1.2m jump height
            sv_stopspeed: 2.5,       // Minimum speed for friction calc
            player_height: 1.8,      // 1.8m tall player
            player_radius: 0.4,      // 0.4m radius
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct PlayerState {
    pub grounded: bool,
    pub wish_jump: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            grounded: false,
            wish_jump: false,
        }
    }
}

/// Input wish direction (normalized horizontal direction player wants to move)
#[derive(Component, Default)]
pub struct WishDir(pub Vec3);

/// Quake-style ground acceleration
pub fn accelerate(
    velocity: Vec3,
    wish_dir: Vec3,
    wish_speed: f32,
    accel: f32,
    dt: f32,
) -> Vec3 {
    let current_speed = velocity.dot(wish_dir);
    let add_speed = wish_speed - current_speed;

    if add_speed <= 0.0 {
        return velocity;
    }

    let accel_speed = (accel * wish_speed * dt).min(add_speed);
    velocity + wish_dir * accel_speed
}

/// Quake-style air acceleration - allows exceeding max speed via strafe jumping
pub fn air_accelerate(
    velocity: Vec3,
    wish_dir: Vec3,
    wish_speed: f32,
    accel: f32,
    dt: f32,
) -> Vec3 {
    // Higher cap allows sharper turns without losing speed
    let wish_speed = wish_speed.min(4.0);

    let current_speed = velocity.dot(wish_dir);
    let add_speed = wish_speed - current_speed;

    if add_speed <= 0.0 {
        return velocity;
    }

    let accel_speed = (accel * wish_speed * dt).min(add_speed);
    let new_vel = velocity + wish_dir * accel_speed;

    // Cap total horizontal speed at 30 m/s
    let horiz_speed = Vec2::new(new_vel.x, new_vel.z).length();
    if horiz_speed > 30.0 {
        let scale = 30.0 / horiz_speed;
        return Vec3::new(new_vel.x * scale, new_vel.y, new_vel.z * scale);
    }

    new_vel
}

/// Apply ground friction
pub fn apply_friction(velocity: Vec3, friction: f32, stop_speed: f32, dt: f32) -> Vec3 {
    let speed = velocity.length();

    if speed < 0.1 {
        return Vec3::ZERO;
    }

    let control = speed.max(stop_speed);
    let drop = control * friction * dt;
    let new_speed = (speed - drop).max(0.0);

    if new_speed > 0.0 {
        velocity * (new_speed / speed)
    } else {
        Vec3::ZERO
    }
}
