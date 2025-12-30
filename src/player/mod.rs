use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildOf;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, WindowFocused};

use crate::GameState;
use crate::level::BoxCollider;

pub mod movement;
pub mod input;

use movement::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementConfig>()
            .add_systems(Startup, (spawn_player, spawn_hud))
            .add_systems(OnEnter(GameState::Playing), grab_cursor)
            .add_systems(OnEnter(GameState::Paused), release_cursor)
            .add_systems(OnEnter(GameState::Menu), release_cursor)
            .add_systems(
                Update,
                (
                    handle_window_focus,
                    player_input,
                    player_look,
                    ground_check,
                    player_movement,
                    apply_gravity,
                    player_collision,
                    apply_velocity,
                    update_view_sway,
                    update_velocity_hud,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera {
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

/// Tracks view effects: bob, sway, landing impact
#[derive(Component)]
pub struct ViewSway {
    pub bob_time: f32,
    pub bob_amount: Vec3,
    pub landing_offset: f32,
    pub velocity_tilt: Vec2,  // Roll and pitch from velocity
    pub prev_grounded: bool,
    pub prev_velocity_y: f32,
}

impl Default for ViewSway {
    fn default() -> Self {
        Self {
            bob_time: 0.0,
            bob_amount: Vec3::ZERO,
            landing_offset: 0.0,
            velocity_tilt: Vec2::ZERO,
            prev_grounded: true,
            prev_velocity_y: 0.0,
        }
    }
}

/// Marker for the viewmodel (arms/weapon)
#[derive(Component)]
pub struct ViewModel;

const MOUSE_SENSITIVITY: f32 = 0.00075;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = MovementConfig::default();

    // Spawn player entity
    let player = commands
        .spawn((
            Player,
            Transform::from_xyz(0.0, config.player_height / 2.0 + 0.1, 10.0),
            Visibility::default(),
            Velocity::default(),
            PlayerState::default(),
            WishDir::default(),
        ))
        .id();

    // Spawn camera as child, offset to eye height
    let eye_offset = config.player_height / 2.0 - 0.1;
    let camera = commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 100.0_f32.to_radians(), // Wide FOV for fast movement feel
            ..default()
        }),
        Transform::from_xyz(0.0, eye_offset, 0.0),
        PlayerCamera::default(),
        ViewSway::default(),
        ChildOf(player),
    )).id();

    // Spawn viewmodel (simple "arms" representation) as child of camera
    // Position: slightly down and forward from camera
    let arm_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.5), // Skin-ish color
        perceptual_roughness: 0.8,
        ..default()
    });

    // Right "arm" - close to camera, short
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.05, 0.05, 0.15))),
        MeshMaterial3d(arm_material.clone()),
        Transform::from_xyz(0.15, -0.12, -0.25),
        ViewModel,
        ChildOf(camera),
    ));

    // Left "arm"
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.05, 0.05, 0.15))),
        MeshMaterial3d(arm_material),
        Transform::from_xyz(-0.15, -0.12, -0.25),
        ViewModel,
        ChildOf(camera),
    ));
}

#[derive(Component)]
pub struct VelocityHud;

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("Speed: 0.0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        VelocityHud,
    ));
}

fn update_velocity_hud(
    player_query: Query<&Velocity, With<Player>>,
    mut hud_query: Query<&mut Text, With<VelocityHud>>,
) {
    let Ok(velocity) = player_query.single() else {
        return;
    };

    let Ok(mut text) = hud_query.single_mut() else {
        return;
    };

    // Calculate horizontal speed (ignore Y for bunny hop display)
    let horiz_speed = Vec2::new(velocity.0.x, velocity.0.z).length();
    **text = format!("Speed: {:.1} m/s", horiz_speed);
}

fn grab_cursor(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn release_cursor(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn handle_window_focus(
    mut focus_events: EventReader<WindowFocused>,
    mut windows: Query<&mut Window>,
) {
    for event in focus_events.read() {
        if let Ok(mut window) = windows.single_mut() {
            if event.focused {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
            } else {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
            }
        }
    }
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut WishDir, &mut PlayerState), With<Player>>,
    camera_query: Query<&PlayerCamera>,
) {
    let Ok((mut wish_dir, mut state)) = player_query.single_mut() else {
        return;
    };

    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Get forward/right vectors from camera yaw (ignore pitch for movement)
    let yaw_rot = Quat::from_rotation_y(camera.yaw);
    let forward = yaw_rot * Vec3::NEG_Z;
    let right = yaw_rot * Vec3::X;

    let mut dir = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        dir += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        dir -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        dir += right;
    }

    // Keep direction horizontal
    dir.y = 0.0;
    wish_dir.0 = if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec3::ZERO
    };

    // Jump input - holding space = continuously want to jump (enables auto-bhop)
    state.wish_jump = keyboard.pressed(KeyCode::Space);
}

fn player_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera)>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let Ok((mut cam_transform, mut camera)) = camera_query.single_mut() else {
        return;
    };

    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    // Update yaw and pitch
    camera.yaw -= delta.x * MOUSE_SENSITIVITY;
    camera.pitch -= delta.y * MOUSE_SENSITIVITY;
    camera.pitch = camera.pitch.clamp(-1.5, 1.5);

    // Apply yaw to player (so they rotate)
    player_transform.rotation = Quat::from_rotation_y(camera.yaw);

    // Apply pitch to camera only (relative to player)
    cam_transform.rotation = Quat::from_rotation_x(camera.pitch);
}

fn ground_check(
    mut query: Query<(&Transform, &mut PlayerState, &Velocity), With<Player>>,
    config: Res<MovementConfig>,
) {
    for (transform, mut state, velocity) in &mut query {
        // Simple ground check: are we at or below ground level?
        let feet_y = transform.translation.y - config.player_height / 2.0;

        // Ground is at y=0, with a small tolerance
        let was_grounded = state.grounded;
        state.grounded = feet_y <= 0.05 && velocity.0.y <= 0.1;

        // If we just landed and weren't trying to jump, clear wish_jump
        if state.grounded && !was_grounded && !state.wish_jump {
            // Landing without jump intent
        }
    }
}

fn player_movement(
    mut query: Query<(&mut Velocity, &mut PlayerState, &WishDir), With<Player>>,
    config: Res<MovementConfig>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut velocity, mut state, wish_dir) in &mut query {
        // Handle jumping - if grounded and holding jump, jump immediately
        // This enables auto-bhop: hold space to jump the frame you land
        if state.grounded && state.wish_jump {
            velocity.0.y = config.sv_jumpspeed;
            state.grounded = false;
            // Don't clear wish_jump - input system handles it based on key state
        }

        // Get horizontal velocity for movement calculations
        let mut horiz_vel = Vec3::new(velocity.0.x, 0.0, velocity.0.z);

        if state.grounded {
            // Ground movement: friction then acceleration
            horiz_vel = apply_friction(
                horiz_vel,
                config.sv_friction,
                config.sv_stopspeed,
                dt,
            );

            if wish_dir.0.length_squared() > 0.0 {
                horiz_vel = accelerate(
                    horiz_vel,
                    wish_dir.0,
                    config.sv_maxspeed,
                    config.sv_accelerate,
                    dt,
                );
            }
        } else {
            // Air movement: no friction, low acceleration
            if wish_dir.0.length_squared() > 0.0 {
                horiz_vel = air_accelerate(
                    horiz_vel,
                    wish_dir.0,
                    config.sv_maxspeed,
                    config.sv_airaccelerate,
                    dt,
                );
            }
        }

        velocity.0.x = horiz_vel.x;
        velocity.0.z = horiz_vel.z;
    }
}

fn apply_gravity(
    mut query: Query<(&mut Velocity, &PlayerState), With<Player>>,
    config: Res<MovementConfig>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut velocity, state) in &mut query {
        if !state.grounded {
            velocity.0.y -= config.sv_gravity * dt;
        }
    }
}

fn player_collision(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    collider_query: Query<(&Transform, &BoxCollider), Without<Player>>,
    config: Res<MovementConfig>,
) {
    for (mut player_transform, mut velocity) in &mut player_query {
        let player_radius = config.player_radius;

        // Floor collision
        let feet_y = player_transform.translation.y - config.player_height / 2.0;
        if feet_y < 0.0 {
            player_transform.translation.y = config.player_height / 2.0;
            if velocity.0.y < 0.0 {
                velocity.0.y = 0.0;
            }
        }

        // Collide with level geometry
        for (collider_transform, collider) in &collider_query {
            let collider_pos = collider_transform.translation;
            let half = collider.half_extents;

            // Player AABB (simplified as a box for XZ, point for Y)
            let player_pos = player_transform.translation;

            // Check XZ collision (2D box vs box)
            let combined_x = half.x + player_radius;
            let combined_z = half.z + player_radius;

            let diff_x = player_pos.x - collider_pos.x;
            let diff_z = player_pos.z - collider_pos.z;

            if diff_x.abs() < combined_x && diff_z.abs() < combined_z {
                // We're colliding in XZ, push out on smallest penetration axis
                let pen_x = combined_x - diff_x.abs();
                let pen_z = combined_z - diff_z.abs();

                if pen_x < pen_z {
                    // Push out on X
                    if diff_x > 0.0 {
                        player_transform.translation.x = collider_pos.x + combined_x;
                        velocity.0.x = velocity.0.x.max(0.0);
                    } else {
                        player_transform.translation.x = collider_pos.x - combined_x;
                        velocity.0.x = velocity.0.x.min(0.0);
                    }
                } else {
                    // Push out on Z
                    if diff_z > 0.0 {
                        player_transform.translation.z = collider_pos.z + combined_z;
                        velocity.0.z = velocity.0.z.max(0.0);
                    } else {
                        player_transform.translation.z = collider_pos.z - combined_z;
                        velocity.0.z = velocity.0.z.min(0.0);
                    }
                }
            }
        }
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * dt;
    }
}

fn update_view_sway(
    player_query: Query<(&Velocity, &PlayerState), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut ViewSway, &PlayerCamera), Without<Player>>,
    mut viewmodel_query: Query<&mut Transform, (With<ViewModel>, Without<Player>, Without<PlayerCamera>)>,
    time: Res<Time>,
) {
    let Ok((velocity, player_state)) = player_query.single() else {
        return;
    };

    let Ok((mut cam_transform, mut sway, camera)) = camera_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let horiz_speed = Vec2::new(velocity.0.x, velocity.0.z).length();

    // === Landing impact ===
    // Detect landing based on velocity change, NOT grounded state
    // This works during bhop where grounded state flips back to false immediately
    // If we were falling significantly and now we're not, we landed
    let was_falling = sway.prev_velocity_y < -2.0;
    let stopped_falling = velocity.0.y > sway.prev_velocity_y + 1.0 || velocity.0.y >= 0.0;

    if was_falling && stopped_falling {
        // Scale impact by fall speed - very gentle
        let base_impact = (sway.prev_velocity_y.abs() / 160.0).clamp(0.0025, 0.01);

        // Reduce further when bhopping (high horizontal speed)
        let bhop_factor = if horiz_speed > 10.0 { 0.5 } else { 1.0 };

        // Blend toward target instead of instant snap for smoother feel
        let target = -base_impact * bhop_factor;
        sway.landing_offset = sway.landing_offset * 0.3 + target * 0.7;
    }
    sway.prev_grounded = player_state.grounded;
    sway.prev_velocity_y = velocity.0.y;

    // Recover from landing impact
    sway.landing_offset = sway.landing_offset * (1.0 - dt * 8.0).max(0.0);

    // === View bob (only when grounded and moving) ===
    if player_state.grounded && horiz_speed > 0.5 {
        // Slower bob for larger step feel
        sway.bob_time += dt * 5.0;

        let bob_x = (sway.bob_time).sin() * 0.003;
        let bob_y = (sway.bob_time * 2.0).sin().abs() * 0.004;

        sway.bob_amount = Vec3::new(bob_x, bob_y, 0.0);
    } else {
        // Smooth return to center when not moving
        sway.bob_amount = sway.bob_amount * (1.0 - dt * 8.0).max(0.0);
    }

    // === Velocity tilt (lean into movement) ===
    // Get velocity relative to camera facing direction
    let forward = Quat::from_rotation_y(camera.yaw) * Vec3::NEG_Z;
    let right = Quat::from_rotation_y(camera.yaw) * Vec3::X;

    let forward_speed = velocity.0.dot(forward);
    let right_speed = velocity.0.dot(right);

    // Target tilt based on velocity - subtle effect
    let target_roll = -(right_speed / 60.0).clamp(-0.03, 0.03);   // Subtle roll
    let target_pitch = (forward_speed / 100.0).clamp(-0.015, 0.015); // Very slight pitch

    // Smooth interpolation
    sway.velocity_tilt.x = sway.velocity_tilt.x + (target_roll - sway.velocity_tilt.x) * dt * 5.0;
    sway.velocity_tilt.y = sway.velocity_tilt.y + (target_pitch - sway.velocity_tilt.y) * dt * 5.0;

    // === Apply to camera transform ===
    // Base position with bob and landing
    let base_y = 0.8; // Eye height offset
    cam_transform.translation = Vec3::new(
        sway.bob_amount.x,
        base_y + sway.bob_amount.y + sway.landing_offset,
        0.0,
    );

    // Apply pitch (from look) + velocity tilt
    cam_transform.rotation = Quat::from_euler(
        EulerRot::XYZ,
        camera.pitch + sway.velocity_tilt.y,
        0.0,
        sway.velocity_tilt.x, // Roll
    );

    // === Apply sway to viewmodel ===
    // Viewmodels react to movement - landing impact more visible on arms
    let vm_offset_x = -sway.velocity_tilt.x * 0.8;
    let vm_offset_y = sway.landing_offset * 8.0 + sway.bob_amount.y * 2.0;

    for mut vm_transform in &mut viewmodel_query {
        // Get base position (set in spawn) and add sway
        let base_x = if vm_transform.translation.x > 0.0 { 0.15 } else { -0.15 };
        vm_transform.translation.x = base_x + vm_offset_x;
        vm_transform.translation.y = -0.12 + vm_offset_y;

        // Subtle rotation with movement
        vm_transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            sway.velocity_tilt.y * 0.2,
            0.0,
            sway.velocity_tilt.x * 0.5,
        );
    }
}
