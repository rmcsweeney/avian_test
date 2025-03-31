mod plugin;
// mod player;

use std::f32::consts::PI;
use bevy::prelude::*;
use avian3d::{prelude::*, math::Scalar};
use bevy::render::camera::RenderTarget;
use plugin::*;

use bevy::window::{CursorGrabMode, PrimaryWindow, WindowRef};
//use bevy::*;
use bevy::app::AppExit;
// use player::*;

fn main() {
    let mut app = App::new();
    app
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), CharacterControllerPlugin))
        .add_systems(Startup, (setup, cursor_grab, setup_observer))
        .add_systems(Update, exit_system)
        .run();
}

fn cursor_grab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();
    primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor_options.visible = false;
}


fn exit_system(mut exit: EventWriter<AppExit>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.any_pressed([KeyCode::Escape]) {
        exit.send(AppExit::Success);
    }
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
            Transform::from_xyz(0.0, 2.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
            PlayerCamera,
            GroundFacingVector::default(),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_euler(EulerRot::YXZ, 0.0, PI/4.0, 0.0))
        ));
    });

    // A cube to move around
    let _cube = commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(3.0, 2.0, 3.0),
    ));

    // Environment (see the `collider_constructors` example for creating colliders from scenes)
    let _level = commands.spawn((
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

    commands.spawn((
        Zoobie,
        NPCControllerBundle::new(Collider::capsule(0.4, 1.0)).with_movement(
            10.0,
            0.92,
            7.0,
            (30.0 as Scalar).to_radians(),
        ),
        Mesh3d(meshes.add(Cone::new(1.0, 2.0))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_xyz(4.0, 2.0, 4.0),
    ));

}

fn setup_observer (mut commands: Commands,) {
    let second_window = commands
    .spawn(Window {
        title: "Second window".to_owned(),
        ..default()
    })
    .id();

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(-5.0, 4.0, -6.0)).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            order: 3,
            target: RenderTarget::Window(WindowRef::Entity(second_window)),
            ..default()
        },
    ));
}


#[derive(Debug, Component)]
pub struct Player;

#[derive(Component)]
pub struct Zoobie;
