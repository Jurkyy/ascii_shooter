use bevy::prelude::*;

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

pub const ARENA_SIZE: f32 = 100.0;

fn spawn_test_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor - big arena for testing bunny hop
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(250.0, 250.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        LevelGeometry,
    ));

    // Walls - create a simple arena
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.4, 0.35),
        perceptual_roughness: 0.8,
        ..default()
    });

    let wall_height = 8.0;
    let wall_thickness = 0.5;

    // North wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ARENA_SIZE * 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, wall_height / 2.0, -ARENA_SIZE),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ARENA_SIZE, wall_height / 2.0, wall_thickness / 2.0) },
    ));

    // South wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ARENA_SIZE * 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, wall_height / 2.0, ARENA_SIZE),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(ARENA_SIZE, wall_height / 2.0, wall_thickness / 2.0) },
    ));

    // East wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, ARENA_SIZE * 2.0))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(ARENA_SIZE, wall_height / 2.0, 0.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(wall_thickness / 2.0, wall_height / 2.0, ARENA_SIZE) },
    ));

    // West wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, ARENA_SIZE * 2.0))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(-ARENA_SIZE, wall_height / 2.0, 0.0),
        LevelGeometry,
        BoxCollider { half_extents: Vec3::new(wall_thickness / 2.0, wall_height / 2.0, ARENA_SIZE) },
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
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, half_height * 2.0, 2.0))),
            MeshMaterial3d(pillar_material.clone()),
            Transform::from_translation(pos),
            LevelGeometry,
            BoxCollider { half_extents: Vec3::new(1.0, half_height, 1.0) },
        ));
    }

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
