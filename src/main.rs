mod plugin;
// mod player;

use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;
use bevy::math::VectorSpace;
use bevy::{prelude::*, input::mouse::AccumulatedMouseMotion, };
use avian3d::{prelude::*, math::Scalar};
use plugin::*;
// use player::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), CharacterControllerPlugin ))
        .add_systems(Startup, setup)
        .add_systems(Update, player_look)
        .run();
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    // Player
    commands.spawn((
        Player,
        CameraSensitivity::default(),
        
        // Mesh3d
        
        Transform::from_xyz(0.0, 1.5, 0.0),
        CharacterControllerBundle::new(Collider::capsule(0.4, 1.0)).with_movement(
            120.0,
            0.92,
            7.0,
            (30.0 as Scalar).to_radians(),
        ),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(2.0),
    )).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 2.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y)
        ));
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_euler(EulerRot::YXZ, 0.0, PI/4.0, 0.0))
        ));
    });

    // A cube to move around
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(3.0, 2.0, 3.0),
    ));

    // Environment (see the `collider_constructors` example for creating colliders from scenes)
    commands.spawn((
        SceneRoot(assets.load("character_controller_demo.glb#Scene0")),
        Transform::from_rotation(Quat::from_rotation_y(-core::f32::consts::PI * 0.5)),
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
        RigidBody::Static,
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            range: 50.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 15.0, 0.0),
    ));

    // Camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-7.0, 9.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));
}



#[derive(Debug, Component, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002),)
    }
}

#[derive(Debug, Component)]
pub struct Player;



fn player_look(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>, 
    mut player: Query<(&mut Transform, &mut CameraSensitivity), Without<Camera3d>>,
    mut camera: Query<(&mut Transform, &mut Camera3d)>

) {
    let Ok((mut player_transform, camera_sensitivity)) = player.get_single_mut() else {
        return;
    };
    let Ok((mut camera_transform, camera3d)) = camera.get_single_mut() else {
        return;
    };
    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        let delta_yaw = -delta.x * camera_sensitivity.x;
        let delta_pitch = -delta.y * camera_sensitivity.y;

        camera_transform.rotate_around(Vec3::ZERO, Quat::from_euler(EulerRot::YXZ, delta_yaw, delta_pitch, 0.0));
        let camera_transform_new = camera_transform.looking_at(Vec3::ZERO, Vec3::Y);
        camera_transform.rotation = camera_transform_new.rotation;

        // (let y, p, r) = camera_transform.rotation.to_euler();
        // camera_transform.rotation.

    }
}