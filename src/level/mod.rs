use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::rendering::AsciiPatternId;

/// Helper to add a quad to mesh data
fn add_quad(
    verts: &mut Vec<[f32; 3]>,
    norms: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3],
    normal: [f32; 3],
) {
    let base = verts.len() as u32;
    verts.extend([p0, p1, p2, p3]);
    norms.extend([normal, normal, normal, normal]);
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Creates a wedge/ramp mesh for slopes in Z direction
/// - `width`: size in X direction
/// - `length`: size in Z direction (the direction the slope runs)
/// - `height_back`: height at the back (-Z end)
/// - `height_front`: height at the front (+Z end)
fn create_ramp_mesh(width: f32, length: f32, height_back: f32, height_front: f32) -> Mesh {
    let hw = width / 2.0;
    let hl = length / 2.0;

    let slope_rise = height_front - height_back;
    let slope_normal = Vec3::new(0.0, length, -slope_rise).normalize();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut norms: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Bottom face
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, 0.0, -hl], [hw, 0.0, -hl], [hw, 0.0, hl], [-hw, 0.0, hl],
        [0.0, -1.0, 0.0]);

    // Top face (sloped)
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, height_back, -hl], [-hw, height_front, hl], [hw, height_front, hl], [hw, height_back, -hl],
        slope_normal.to_array());

    // Back face
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, 0.0, -hl], [-hw, height_back, -hl], [hw, height_back, -hl], [hw, 0.0, -hl],
        [0.0, 0.0, -1.0]);

    // Front face
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [hw, 0.0, hl], [hw, height_front, hl], [-hw, height_front, hl], [-hw, 0.0, hl],
        [0.0, 0.0, 1.0]);

    // Left face
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, 0.0, hl], [-hw, height_front, hl], [-hw, height_back, -hl], [-hw, 0.0, -hl],
        [-1.0, 0.0, 0.0]);

    // Right face
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [hw, 0.0, -hl], [hw, height_back, -hl], [hw, height_front, hl], [hw, 0.0, hl],
        [1.0, 0.0, 0.0]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, norms);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Creates a ramp mesh for X-direction slopes
fn create_ramp_mesh_x(width: f32, length: f32, height_left: f32, height_right: f32) -> Mesh {
    let hw = width / 2.0;
    let hl = length / 2.0;

    let slope_rise = height_right - height_left;
    let slope_normal = Vec3::new(-slope_rise, width, 0.0).normalize();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut norms: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Bottom
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, 0.0, -hl], [hw, 0.0, -hl], [hw, 0.0, hl], [-hw, 0.0, hl],
        [0.0, -1.0, 0.0]);

    // Top (sloped in X)
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, height_left, -hl], [-hw, height_left, hl], [hw, height_right, hl], [hw, height_right, -hl],
        slope_normal.to_array());

    // Front (+Z)
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, 0.0, hl], [-hw, height_left, hl], [hw, height_right, hl], [hw, 0.0, hl],
        [0.0, 0.0, 1.0]);

    // Back (-Z)
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [hw, 0.0, -hl], [hw, height_right, -hl], [-hw, height_left, -hl], [-hw, 0.0, -hl],
        [0.0, 0.0, -1.0]);

    // Left (-X)
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [-hw, 0.0, -hl], [-hw, height_left, -hl], [-hw, height_left, hl], [-hw, 0.0, hl],
        [-1.0, 0.0, 0.0]);

    // Right (+X)
    add_quad(&mut verts, &mut norms, &mut uvs, &mut indices,
        [hw, 0.0, hl], [hw, height_right, hl], [hw, height_right, -hl], [hw, 0.0, -hl],
        [1.0, 0.0, 0.0]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, norms);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_level);
    }
}

#[derive(Component)]
pub struct LevelGeometry;

/// Box collider for level geometry - stores half-extents
#[derive(Component)]
pub struct BoxCollider {
    pub half_extents: Vec3,
}

/// Marker for wall colliders that block player/enemy movement
/// (Floors only block projectiles, not movement)
#[derive(Component)]
pub struct WallCollider;

/// Marker for the main ground floor (excluded from platform collision)
#[derive(Component)]
pub struct GroundFloor;

/// Slope component for angled floor surfaces
/// The slope rises in the direction specified, starting from base_height
#[derive(Component)]
pub struct Slope {
    /// Direction the slope rises (normalized, in XZ plane)
    pub direction: Vec2,
    /// Height rise per unit distance in the slope direction
    pub rise_per_unit: f32,
}

impl Slope {
    /// Create a slope that rises in the +Z direction
    pub fn rising_z(rise_per_unit: f32) -> Self {
        Self {
            direction: Vec2::new(0.0, 1.0),
            rise_per_unit,
        }
    }

    /// Create a slope that rises in the -Z direction
    pub fn falling_z(rise_per_unit: f32) -> Self {
        Self {
            direction: Vec2::new(0.0, -1.0),
            rise_per_unit,
        }
    }

    /// Create a slope that rises in the +X direction
    pub fn rising_x(rise_per_unit: f32) -> Self {
        Self {
            direction: Vec2::new(1.0, 0.0),
            rise_per_unit,
        }
    }

    /// Create a slope that rises in the -X direction
    pub fn falling_x(rise_per_unit: f32) -> Self {
        Self {
            direction: Vec2::new(-1.0, 0.0),
            rise_per_unit,
        }
    }

    /// Calculate the ground height at a given world position
    pub fn height_at(&self, slope_center: Vec3, half_extents: Vec3, world_pos: Vec3) -> f32 {
        // Project position onto slope direction relative to slope center
        let relative_x = world_pos.x - slope_center.x;
        let relative_z = world_pos.z - slope_center.z;
        let relative_pos = Vec2::new(relative_x, relative_z);

        // Distance along the slope direction from center
        let distance_along = relative_pos.dot(self.direction);

        // Base height is at the center, adjusted by distance along slope
        let base_top = slope_center.y + half_extents.y;
        base_top + distance_along * self.rise_per_unit
    }
}

pub const ARENA_SIZE: f32 = 100.0;

fn spawn_test_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor - big arena for testing bunny hop (Standard ASCII pattern)
    // Has a thin BoxCollider for projectile collision detection
    // GroundFloor marker excludes it from player platform collision (uses y=0 check instead)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(250.0, 250.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(125.0, 0.5, 125.0) }, // Thin floor collider
        GroundFloor, // Excluded from player floor collision
        AsciiPatternId::standard(),
    ));

    // Walls - create a simple arena
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.4, 0.35),
        perceptual_roughness: 0.8,
        ..default()
    });

    let wall_height = 8.0;
    let wall_thickness = 0.5;

    // North wall (Blocks ASCII pattern)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ARENA_SIZE * 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, wall_height / 2.0, -ARENA_SIZE),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ARENA_SIZE, wall_height / 2.0, wall_thickness / 2.0) },
        WallCollider,
        AsciiPatternId::blocks(),
    ));

    // South wall (Blocks ASCII pattern)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ARENA_SIZE * 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, wall_height / 2.0, ARENA_SIZE),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ARENA_SIZE, wall_height / 2.0, wall_thickness / 2.0) },
        WallCollider,
        AsciiPatternId::blocks(),
    ));

    // East wall (Blocks ASCII pattern)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, ARENA_SIZE * 2.0))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(ARENA_SIZE, wall_height / 2.0, 0.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(wall_thickness / 2.0, wall_height / 2.0, ARENA_SIZE) },
        WallCollider,
        AsciiPatternId::blocks(),
    ));

    // West wall (Blocks ASCII pattern)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, ARENA_SIZE * 2.0))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(-ARENA_SIZE, wall_height / 2.0, 0.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(wall_thickness / 2.0, wall_height / 2.0, ARENA_SIZE) },
        WallCollider,
        AsciiPatternId::blocks(),
    ));

    // Some pillars/obstacles - spread out in the larger arena
    let pillar_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.55, 0.5),
        perceptual_roughness: 0.7,
        ..default()
    });

    let pillar_positions = [
        Vec3::new(-40.0, 2.0, -40.0),
        Vec3::new(40.0, 2.0, -40.0),
        Vec3::new(-40.0, 2.0, 40.0),
        Vec3::new(40.0, 2.0, 40.0),
        Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(-70.0, 3.0, 0.0),
        Vec3::new(70.0, 3.0, 0.0),
        Vec3::new(0.0, 3.0, -70.0),
        Vec3::new(0.0, 3.0, 70.0),
    ];

    for pos in pillar_positions {
        let half_height = pos.y;
        let pillar_top = pos.y + half_height;

        // Pillar body (blocks horizontal movement)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, half_height * 2.0, 2.0))),
            MeshMaterial3d(pillar_material.clone()),
            Transform::from_translation(pos),
            LevelGeometry,
            BoxCollider { half_extents: Vec3::new(1.0, half_height, 1.0) },
            WallCollider,
            AsciiPatternId::slashes(),
        ));

        // Pillar top (floor surface for standing on)
        commands.spawn((
            Transform::from_xyz(pos.x, pillar_top, pos.z),
            BoxCollider { half_extents: Vec3::new(1.0, 0.1, 1.0) },
            // No WallCollider, no mesh - just collision for floor detection
        ));
    }

    // Pattern showcase area - 4 cubes near spawn to compare all patterns
    // All cubes have emissive properties so patterns are clearly visible
    let showcase_size = 2.5;
    let showcase_height = 3.0;
    let showcase_y = showcase_height / 2.0;
    let showcase_z = -8.0;
    let showcase_spacing = 4.0;

    // Pattern 0: Standard - white/grey
    let standard_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        emissive: LinearRgba::rgb(0.3, 0.3, 0.3),
        perceptual_roughness: 0.5,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(showcase_size, showcase_height, showcase_size))),
        MeshMaterial3d(standard_material),
        Transform::from_xyz(-showcase_spacing * 1.5, showcase_y, showcase_z),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::splat(showcase_size / 2.0) },
        WallCollider,
        AsciiPatternId::standard(),
    ));

    // Pattern 1: Blocks - orange
    let blocks_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.6, 0.2),
        emissive: LinearRgba::rgb(0.3, 0.15, 0.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(showcase_size, showcase_height, showcase_size))),
        MeshMaterial3d(blocks_material),
        Transform::from_xyz(-showcase_spacing * 0.5, showcase_y, showcase_z),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::splat(showcase_size / 2.0) },
        WallCollider,
        AsciiPatternId::blocks(),
    ));

    // Pattern 2: Slashes - blue
    let slashes_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.6, 1.0),
        emissive: LinearRgba::rgb(0.1, 0.15, 0.3),
        perceptual_roughness: 0.5,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(showcase_size, showcase_height, showcase_size))),
        MeshMaterial3d(slashes_material),
        Transform::from_xyz(showcase_spacing * 0.5, showcase_y, showcase_z),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::splat(showcase_size / 2.0) },
        WallCollider,
        AsciiPatternId::slashes(),
    ));

    // Pattern 3: Binary - green
    let binary_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 1.0, 0.4),
        emissive: LinearRgba::rgb(0.0, 0.3, 0.1),
        perceptual_roughness: 0.4,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(showcase_size, showcase_height, showcase_size))),
        MeshMaterial3d(binary_material),
        Transform::from_xyz(showcase_spacing * 1.5, showcase_y, showcase_z),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::splat(showcase_size / 2.0) },
        WallCollider,
        AsciiPatternId::binary(),
    ));

    // Pattern 4: Matrix Cycle - dark green/cyan (animated cycling characters)
    let matrix_cycle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.8, 0.5),
        emissive: LinearRgba::rgb(0.0, 0.4, 0.2),
        perceptual_roughness: 0.3,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(showcase_size, showcase_height, showcase_size))),
        MeshMaterial3d(matrix_cycle_material),
        Transform::from_xyz(showcase_spacing * 2.5, showcase_y, showcase_z),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::splat(showcase_size / 2.0) },
        WallCollider,
        AsciiPatternId::matrix_cycle(),
    ));

    // Pattern 5: Matrix Fall - bright green (true falling rain effect)
    let matrix_fall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.3),
        emissive: LinearRgba::rgb(0.0, 0.5, 0.1),
        perceptual_roughness: 0.2,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(showcase_size, showcase_height, showcase_size))),
        MeshMaterial3d(matrix_fall_material),
        Transform::from_xyz(showcase_spacing * 3.5, showcase_y, showcase_z),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::splat(showcase_size / 2.0) },
        WallCollider,
        AsciiPatternId::matrix_fall(),
    ));

    // Raised platform to test floor collision at different elevations
    // Has BoxCollider (for projectile collision) but NO WallCollider (doesn't block horizontal movement)
    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.35, 0.5),
        emissive: LinearRgba::rgb(0.1, 0.05, 0.15),
        perceptual_roughness: 0.6,
        ..default()
    });

    // Main raised platform - can walk up ramp or jump onto
    let platform_width = 20.0;
    let platform_depth = 20.0;
    let platform_height = 0.5;
    let platform_y = 3.0; // Elevated 3 units
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(platform_width, platform_height, platform_depth))),
        MeshMaterial3d(platform_material.clone()),
        Transform::from_xyz(30.0, platform_y, 30.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(platform_width / 2.0, platform_height / 2.0, platform_depth / 2.0) },
        // NO WallCollider - this is a floor, not a wall
        AsciiPatternId::binary(),
    ));

    // Stairs leading up to the platform (replacing ramp since AABB can't handle rotation)
    let stair_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.4, 0.55),
        perceptual_roughness: 0.7,
        ..default()
    });

    // Create steps leading up to the platform (which is at y=3.0, top at 3.25)
    // Each step is max 0.6 units higher than the previous (within 0.7 step-up range)
    let step_width = 8.0;
    let step_depth = 2.5;
    let step_thickness = 0.3;
    // Step tops: 0.4, 0.95, 1.5, 2.05, 2.6, 3.15 (each ~0.55 higher)
    let step_heights = [0.25, 0.8, 1.35, 1.9, 2.45, 3.0];
    let step_z_positions = [8.0, 11.0, 14.0, 17.0, 20.0, 23.0];

    for (&height, &z_pos) in step_heights.iter().zip(step_z_positions.iter()) {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(step_width, step_thickness, step_depth))),
            MeshMaterial3d(stair_material.clone()),
            Transform::from_xyz(30.0, height, z_pos),
            LevelGeometry,
            BoxCollider { half_extents: Vec3::new(step_width / 2.0, step_thickness / 2.0, step_depth / 2.0) },
            // NO WallCollider - stairs are floor surfaces
            AsciiPatternId::slashes(),
        ));
    }

    // === SLOPE RAMPS ===
    let slope_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.7, 0.5),
        emissive: LinearRgba::rgb(0.05, 0.15, 0.05),
        perceptual_roughness: 0.5,
        ..default()
    });

    // Ramp going up in +Z direction (next to stairs, leads to same platform)
    // Ramp is 15 units long, rises from 0 to 3 (slope = 0.2 rise per unit)
    let ramp1_length = 15.0;
    let ramp1_width = 6.0;
    let ramp1_height_back = 0.0;
    let ramp1_height_front = 3.0;
    let ramp1_center_height = (ramp1_height_back + ramp1_height_front) / 2.0;
    commands.spawn((
        Mesh3d(meshes.add(create_ramp_mesh(ramp1_width, ramp1_length, ramp1_height_back, ramp1_height_front))),
        MeshMaterial3d(slope_material.clone()),
        Transform::from_xyz(40.0, 0.0, 15.5), // Base at ground level
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ramp1_width / 2.0, ramp1_center_height, ramp1_length / 2.0) },
        Slope::rising_z(0.2), // Rises 0.2 units per Z unit
        AsciiPatternId::standard(),
    ));

    // Ramp going up in +X direction (different area)
    // 12 units wide, rises from 0 to 3 (0.25 per unit)
    let ramp2_width = 12.0;
    let ramp2_depth = 6.0;
    let ramp2_height_left = 0.0;
    let ramp2_height_right = 3.0;
    let ramp2_center_height = (ramp2_height_left + ramp2_height_right) / 2.0;
    commands.spawn((
        Mesh3d(meshes.add(create_ramp_mesh_x(ramp2_width, ramp2_depth, ramp2_height_left, ramp2_height_right))),
        MeshMaterial3d(slope_material.clone()),
        Transform::from_xyz(-50.0, 0.0, -30.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ramp2_width / 2.0, ramp2_center_height, ramp2_depth / 2.0) },
        Slope::rising_x(0.25), // Rises 0.25 units per X unit
        AsciiPatternId::standard(),
    ));

    // Platform at top of X-direction ramp
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 0.5, 8.0))),
        MeshMaterial3d(platform_material.clone()),
        Transform::from_xyz(-42.0, 3.0, -30.0), // Adjusted to match ramp top
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(4.0, 0.25, 4.0) },
        AsciiPatternId::binary(),
    ));

    // Steeper ramp (for variety) - rises 4 units over 10
    let ramp3_width = 5.0;
    let ramp3_length = 10.0;
    let ramp3_height_back = 0.0;
    let ramp3_height_front = 4.0;
    let ramp3_center_height = (ramp3_height_back + ramp3_height_front) / 2.0;
    commands.spawn((
        Mesh3d(meshes.add(create_ramp_mesh(ramp3_width, ramp3_length, ramp3_height_back, ramp3_height_front))),
        MeshMaterial3d(slope_material),
        Transform::from_xyz(-60.0, 0.0, 20.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ramp3_width / 2.0, ramp3_center_height, ramp3_length / 2.0) },
        Slope::rising_z(0.4), // Steeper: 0.4 rise per unit
        AsciiPatternId::slashes(),
    ));

    // Second smaller platform at different height
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 0.5, 10.0))),
        MeshMaterial3d(platform_material),
        Transform::from_xyz(-30.0, 5.0, 30.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(5.0, 0.25, 5.0) },
        AsciiPatternId::matrix_cycle(),
    ));

    // Multiple lights for the larger arena
    let light_positions = [
        Vec3::new(0.0, 15.0, 0.0),
        Vec3::new(-50.0, 15.0, -50.0),
        Vec3::new(50.0, 15.0, -50.0),
        Vec3::new(-50.0, 15.0, 50.0),
        Vec3::new(50.0, 15.0, 50.0),
    ];

    for pos in light_positions {
        commands.spawn((
            PointLight {
                intensity: 1000000.0,
                shadows_enabled: true,
                range: 80.0,
                ..default()
            },
            Transform::from_translation(pos),
        ));
    }

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.7, 0.75, 0.8),
        brightness: 300.0,
        ..default()
    });
}
