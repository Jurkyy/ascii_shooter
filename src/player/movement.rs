use bevy::prelude::*;

/// CS Surf/Bhop server style movement constants
/// Inspired by CS:S/CS:GO community bhop and surf servers
#[derive(Resource)]
pub struct MovementConfig {
    pub sv_maxspeed: f32,           // Max ground speed (units/sec)
    pub sv_accelerate: f32,         // Ground acceleration
    pub sv_airaccelerate: f32,      // Air acceleration (high for surf/bhop)
    pub sv_friction: f32,           // Ground friction
    pub sv_gravity: f32,            // Gravity (units/sec^2)
    pub sv_jumpspeed: f32,          // Jump velocity
    pub sv_stopspeed: f32,          // Speed below which friction stops you instantly
    pub sv_air_wishspeed_cap: f32,  // Wish speed cap in air (controls strafe tightness)
    pub sv_air_speed_cap: f32,      // Hard cap on air speed
    pub player_height: f32,         // Player capsule height
    pub player_radius: f32,         // Player capsule radius
}

impl Default for MovementConfig {
    fn default() -> Self {
        // CS Surf/Bhop style: smooth air control, speed gain through strafing
        // Reference: bhop servers use high sv_airaccelerate (100+) and sv_enablebunnyhopping 1
        Self {
            sv_maxspeed: 7.5,           // Base ground speed
            sv_accelerate: 6.0,         // Moderate ground accel (less twitchy than Quake)
            sv_airaccelerate: 12.0,     // High air accel for responsive strafing
            sv_friction: 5.0,           // Higher friction = snappier ground stops
            sv_gravity: 12.0,           // Lower gravity for more hangtime (surf feel)
            sv_jumpspeed: 6.2,          // Good jump height for bhop chains
            sv_stopspeed: 1.8,          // Threshold for instant stop
            sv_air_wishspeed_cap: 1.5,  // Tighter than Quake = smoother curves
            sv_air_speed_cap: 25.0,     // Soft cap on max speed
            player_height: 1.8,
            player_radius: 0.4,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct PlayerState {
    pub grounded: bool,
    pub wish_jump: bool,
    /// Current ground height (y position of the floor surface the player is on or above)
    pub ground_height: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            grounded: false,
            wish_jump: false,
            ground_height: 0.0,
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

/// CS Surf/Bhop style air acceleration - allows speed gain through strafing
/// The key mechanic: wish_speed is capped low for tighter turning curves,
/// but acceleration uses full wish_speed, enabling speed gain when strafing
/// perpendicular to velocity.
pub fn air_accelerate(
    velocity: Vec3,
    wish_dir: Vec3,
    wish_speed: f32,
    accel: f32,
    air_wishspeed_cap: f32,
    air_speed_cap: f32,
    dt: f32,
) -> Vec3 {
    // Cap wish_speed for add_speed calculation - lower = smoother, tighter curves
    // This is what gives surf its smooth, flowing feel vs Quake's twitchiness
    let capped_wish_speed = wish_speed.min(air_wishspeed_cap);

    let current_speed = velocity.dot(wish_dir);
    let add_speed = capped_wish_speed - current_speed;

    if add_speed <= 0.0 {
        return velocity;
    }

    // Use full wish_speed for acceleration - this enables speed gain through strafing
    // When strafing perpendicular to velocity, current_speed ≈ 0, so add_speed > 0
    // and you gain velocity in the strafe direction
    let accel_speed = (accel * wish_speed * dt).min(add_speed);
    let new_vel = velocity + wish_dir * accel_speed;

    // Soft cap on horizontal speed - allows high speeds but with diminishing returns
    let horiz_speed = Vec2::new(new_vel.x, new_vel.z).length();
    if horiz_speed > air_speed_cap {
        let scale = air_speed_cap / horiz_speed;
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

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec3_approx_eq(a: Vec3, b: Vec3) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z)
    }

    // ==================== MovementConfig Tests ====================

    #[test]
    fn test_movement_config_defaults() {
        let config = MovementConfig::default();

        // CS Surf/Bhop style values
        assert!(approx_eq(config.sv_maxspeed, 7.5));
        assert!(approx_eq(config.sv_accelerate, 6.0));
        assert!(approx_eq(config.sv_airaccelerate, 12.0));
        assert!(approx_eq(config.sv_friction, 5.0));
        assert!(approx_eq(config.sv_gravity, 12.0));
        assert!(approx_eq(config.sv_jumpspeed, 6.2));
        assert!(approx_eq(config.sv_air_wishspeed_cap, 1.5));
        assert!(approx_eq(config.sv_air_speed_cap, 25.0));
        assert!(approx_eq(config.player_height, 1.8));
        assert!(approx_eq(config.player_radius, 0.4));
    }

    // ==================== Accelerate Tests ====================

    #[test]
    fn test_accelerate_from_standstill() {
        let velocity = Vec3::ZERO;
        let wish_dir = Vec3::new(0.0, 0.0, -1.0); // Forward
        let wish_speed = 8.0;
        let accel = 10.0;
        let dt = 0.016; // ~60fps

        let result = accelerate(velocity, wish_dir, wish_speed, accel, dt);

        // Should accelerate in wish direction
        assert!(result.z < 0.0);
        assert!(approx_eq(result.x, 0.0));
        assert!(approx_eq(result.y, 0.0));
    }

    #[test]
    fn test_accelerate_already_at_max_speed() {
        let velocity = Vec3::new(0.0, 0.0, -8.0); // Already at max speed
        let wish_dir = Vec3::new(0.0, 0.0, -1.0);
        let wish_speed = 8.0;
        let accel = 10.0;
        let dt = 0.016;

        let result = accelerate(velocity, wish_dir, wish_speed, accel, dt);

        // Should not accelerate further
        assert!(vec3_approx_eq(result, velocity));
    }

    #[test]
    fn test_accelerate_above_max_speed_no_change() {
        let velocity = Vec3::new(0.0, 0.0, -15.0); // Above max
        let wish_dir = Vec3::new(0.0, 0.0, -1.0);
        let wish_speed = 8.0;
        let accel = 10.0;
        let dt = 0.016;

        let result = accelerate(velocity, wish_dir, wish_speed, accel, dt);

        // Should not change velocity when already above max
        assert!(vec3_approx_eq(result, velocity));
    }

    #[test]
    fn test_accelerate_perpendicular_strafe() {
        let velocity = Vec3::new(0.0, 0.0, -8.0); // Moving forward at max
        let wish_dir = Vec3::new(1.0, 0.0, 0.0);  // Strafe right
        let wish_speed = 8.0;
        let accel = 10.0;
        let dt = 0.016;

        let result = accelerate(velocity, wish_dir, wish_speed, accel, dt);

        // Should add strafe velocity
        assert!(result.x > 0.0);
        // Forward velocity unchanged
        assert!(approx_eq(result.z, velocity.z));
    }

    // ==================== Air Accelerate Tests ====================

    #[test]
    fn test_air_accelerate_basic() {
        let velocity = Vec3::new(0.0, 0.0, -5.0);
        let wish_dir = Vec3::new(1.0, 0.0, 0.0); // Strafe
        let wish_speed = 7.5;
        let accel = 12.0;
        let air_wishspeed_cap = 1.5;
        let air_speed_cap = 25.0;
        let dt = 0.016;

        let result = air_accelerate(velocity, wish_dir, wish_speed, accel, air_wishspeed_cap, air_speed_cap, dt);

        // Should gain speed in strafe direction
        assert!(result.x > 0.0);
    }

    #[test]
    fn test_air_accelerate_speed_cap() {
        let velocity = Vec3::new(20.0, 0.0, -15.0); // High horizontal speed
        let wish_dir = Vec3::new(1.0, 0.0, 0.0);
        let wish_speed = 7.5;
        let accel = 12.0;
        let air_wishspeed_cap = 1.5;
        let air_speed_cap = 25.0;
        let dt = 0.016;

        let result = air_accelerate(velocity, wish_dir, wish_speed, accel, air_wishspeed_cap, air_speed_cap, dt);

        // Should cap at air_speed_cap horizontal
        let horiz_speed = Vec2::new(result.x, result.z).length();
        assert!(horiz_speed <= air_speed_cap + EPSILON);
    }

    #[test]
    fn test_air_accelerate_preserves_vertical() {
        let velocity = Vec3::new(5.0, -10.0, 5.0); // Falling
        let wish_dir = Vec3::new(1.0, 0.0, 0.0);
        let wish_speed = 7.5;
        let accel = 12.0;
        let air_wishspeed_cap = 1.5;
        let air_speed_cap = 25.0;
        let dt = 0.016;

        let result = air_accelerate(velocity, wish_dir, wish_speed, accel, air_wishspeed_cap, air_speed_cap, dt);

        // Y velocity should be unchanged
        assert!(approx_eq(result.y, velocity.y));
    }

    // ==================== Friction Tests ====================

    #[test]
    fn test_friction_reduces_speed() {
        let velocity = Vec3::new(0.0, 0.0, -8.0);
        let friction = 6.0;
        let stop_speed = 2.5;
        let dt = 0.016;

        let result = apply_friction(velocity, friction, stop_speed, dt);

        // Speed should decrease
        assert!(result.length() < velocity.length());
        // Direction should be preserved
        assert!(result.z < 0.0);
    }

    #[test]
    fn test_friction_stops_slow_movement() {
        let velocity = Vec3::new(0.0, 0.0, -0.05); // Very slow
        let friction = 6.0;
        let stop_speed = 2.5;
        let dt = 0.016;

        let result = apply_friction(velocity, friction, stop_speed, dt);

        // Should stop completely
        assert!(vec3_approx_eq(result, Vec3::ZERO));
    }

    #[test]
    fn test_friction_preserves_direction() {
        let velocity = Vec3::new(3.0, 0.0, -4.0); // Diagonal movement
        let friction = 6.0;
        let stop_speed = 2.5;
        let dt = 0.016;

        let result = apply_friction(velocity, friction, stop_speed, dt);

        // Direction should be same (normalized)
        let orig_dir = velocity.normalize();
        let new_dir = result.normalize();
        assert!(vec3_approx_eq(orig_dir, new_dir));
    }

    #[test]
    fn test_friction_zero_velocity() {
        let velocity = Vec3::ZERO;
        let friction = 6.0;
        let stop_speed = 2.5;
        let dt = 0.016;

        let result = apply_friction(velocity, friction, stop_speed, dt);

        assert!(vec3_approx_eq(result, Vec3::ZERO));
    }

    // ==================== PlayerState Tests ====================

    #[test]
    fn test_player_state_default() {
        let state = PlayerState::default();

        assert!(!state.grounded);
        assert!(!state.wish_jump);
    }

    // ==================== Velocity Tests ====================

    #[test]
    fn test_velocity_default() {
        let vel = Velocity::default();

        assert!(vec3_approx_eq(vel.0, Vec3::ZERO));
    }

    // ==================== Integration-style Tests ====================

    #[test]
    fn test_bunny_hop_gains_speed() {
        // Simulate a bunny hop: strafe perpendicular to velocity direction
        // Key insight: you gain speed by strafing ~90 degrees from movement direction
        // while turning your view to change the velocity angle
        let mut velocity = Vec3::new(0.0, 0.0, -8.0);
        let accel = 10.0; // Realistic accel value
        let dt = 0.016;

        let initial_speed = Vec2::new(velocity.x, velocity.z).length();

        // Strafe RIGHT (perpendicular to forward velocity) - this is proper bhop technique
        // The wish_dir should be nearly perpendicular to current velocity
        for _ in 0..30 {
            let wish_dir = Vec3::new(1.0, 0.0, 0.0); // Pure right strafe
            velocity = air_accelerate(velocity, wish_dir, 8.0, accel, dt);
        }

        let final_speed = Vec2::new(velocity.x, velocity.z).length();

        // Speed should increase (bunny hop effect)
        assert!(final_speed > initial_speed);
    }

    #[test]
    fn test_ground_movement_caps_at_maxspeed() {
        let mut velocity = Vec3::ZERO;
        let wish_dir = Vec3::new(0.0, 0.0, -1.0);
        let wish_speed = 8.0;
        let accel = 10.0;
        let dt = 0.016;

        // Accelerate for many frames
        for _ in 0..1000 {
            velocity = accelerate(velocity, wish_dir, wish_speed, accel, dt);
        }

        let speed = velocity.length();

        // Should cap at wish_speed
        assert!(speed <= wish_speed + EPSILON);
    }
}
