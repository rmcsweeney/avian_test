use std::f32::consts::{FRAC_PI_3, FRAC_PI_4};

use bevy::{input::{keyboard, mouse::AccumulatedMouseMotion}, prelude::*};
use avian3d::{math::{Quaternion, Scalar, Vector, Vector2, FRAC_PI_2, PI}, prelude::*};pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>().add_systems(
            Update,
            (
                player_look,
                keyboard_input,
                update_grounded,
                movement,
                apply_movement_damping,
            ).chain(),
        );
    }
}

#[derive(Event)]
pub enum MovementAction {
    Move(Vector2),
    Jump,
}

#[derive(Component)]
pub struct CharacterController;

#[derive(Component)]
pub struct NPCController;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

#[derive(Component)]
pub struct MovementAcceleration(Scalar);

#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

#[derive(Component)]
pub struct JumpImpulse(Scalar);

#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

#[derive(Bundle)]
pub struct NPCControllerBundle {
    npc_controller: NPCController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self { 
            acceleration: MovementAcceleration(acceleration), 
            damping: MovementDampingFactor(damping), 
            jump_impulse: JumpImpulse(jump_impulse), 
            max_slope_angle: MaxSlopeAngle(max_slope_angle), 
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO, 
                Quaternion::default(), 
                Dir3::NEG_Y
            ).with_max_distance(0.2),
            locked_axes: LockedAxes::from_bits(0b000_101),
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}

impl NPCControllerBundle {
    pub fn new(collider: Collider) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            npc_controller: NPCController,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO, 
                Quaternion::default(), 
                Dir3::NEG_Y
            ).with_max_distance(0.2),
            locked_axes: LockedAxes::from_bits(0b000_101),
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}

#[derive(Debug, Component, Deref, DerefMut)]
pub struct CameraSensitivity(Vec2);

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct GroundFacingVector(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002),)
    }
}

impl Default for GroundFacingVector {
    fn default() -> Self {
        Self(Vec2::new(0.0, 0.0),)
    }
}

fn player_look(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>, 
    mut player: Query<(&mut CameraSensitivity), Without<Camera3d>>,
    mut camera: Query<(&mut Transform, &mut GroundFacingVector), With<PlayerCamera>>

) {
    let Ok(camera_sensitivity) = player.get_single_mut() else {
        return;
    };
    let Ok((mut camera_transform, mut ground_facing_vector)) = camera.get_single_mut() else {
        return;
    };
    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        //TODO: below pitch clamping is boken
        let delta_yaw = -delta.x * camera_sensitivity.x;
        let mut delta_pitch = -delta.y * camera_sensitivity.y;
        println!("Pitch before clamp: {}", delta_pitch);
        let (_yaw, pitch, _roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        const PITCH_LIMIT: f32 = FRAC_PI_4;
        let new_pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        println!("Current pitch: {} New pitch: {}", pitch, new_pitch);
        if new_pitch > PITCH_LIMIT {
            delta_pitch = PITCH_LIMIT - pitch;
        } else if new_pitch < -PITCH_LIMIT {
            delta_pitch = -PITCH_LIMIT - pitch;
        }
        println!("Pitch after clamp: {}", delta_pitch);

        camera_transform.rotate_around(Vec3::ZERO, Quat::from_euler(EulerRot::YXZ, delta_yaw, delta_pitch, 0.0));
        let camera_transform_new = camera_transform.looking_at(Vec3::ZERO, Vec3::Y);
        camera_transform.rotation = camera_transform_new.rotation;

        // (let y, p, r) = camera_transform.rotation.to_euler();
        // camera_transform.rotation.
        ground_facing_vector.0 = Vec2::new(-camera_transform.forward().x, camera_transform.forward().z).normalize();
    }
}

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera: Query<&GroundFacingVector, With<Camera3d>>,
    // ccb: Query<&CharacterControllerBundle, With<Player>>
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let mut direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);

    
    let Ok(ground_facing_vector) = camera.get_single() else {
        println!("also fuck this");
        return;
    };

    //let ground_facing_vector = Vec2::new(-camera_transform.forward().x, camera_transform.forward().z).normalize();

    direction = -direction.y * ground_facing_vector.0 + direction.x * Vec2::new(-ground_facing_vector.0.y, ground_facing_vector.0.x);

    if direction != Vector2::ZERO {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }
}

// fn do_movement()

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        Has<Grounded>
    ), 
        With<CharacterController>>
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs();

    for event in movement_event_reader.read() {
        for (movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in
            &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    linear_velocity.x += direction.x * movement_acceleration.0 * delta_time;
                    linear_velocity.z -= direction.y * movement_acceleration.0 * delta_time;
                }
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
            }
        }
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}

#[derive(Debug, Component)]
pub struct Player;

#[derive(Component)]
pub struct Zoobie;

fn zoobie_move(
    mut movement_event_writer: EventWriter<MovementAction>,
    mut player_transform: Query<&Transform, (With<Player>, Without<Zoobie>)>,
    mut zoobies: Query<&Transform, (With<Zoobie>, Without<Player>)>,
) {
    let aggro = true;
    let horizontal = aggro as i8;

    // let Ok(mut zoobie_transform) = zoobies.get_mut(entity) {

    // }

    // let mut direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);
    // direction = -direction.y * ground_facing_vector.0 + direction.x * Vec2::new(-ground_facing_vector.0.y, ground_facing_vector.0.x);

    // if direction != Vector2::ZERO {
    //     movement_event_writer.send(MovementAction::Move(direction));
    // }
}