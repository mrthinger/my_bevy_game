use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>().add_systems(
            Update,
            (
                keyboard_input,
                gamepad_input,
                update_grounded,
                movement,
                apply_movement_damping,
            )
                .chain(),
        );
    }
}

/// An event sent for a movement input action.
#[derive(Event)]
pub enum MovementAction {
    Move(Scalar),
    Jump,
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController {
    wall_jumps: u32,
    last_wall: Option<Grounded>,
}

/// A marker component indicating that an entity is on the ground.
#[derive(Component, PartialEq, Clone)]
pub enum Grounded {
    None,
    Ground,
    LeftWall,
    RightWall,
}
/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

#[derive(Component)]
pub struct ShapeCastShape(Collider);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    caster_shape: ShapeCastShape,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    grounded: Grounded,
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
            grounded: Grounded::None,
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
            character_controller: CharacterController {
                wall_jumps: 0,
                last_wall: None,
            },
            rigid_body: RigidBody::Dynamic,
            collider,
            caster_shape: ShapeCastShape(caster_shape),
            locked_axes: LockedAxes::ROTATION_LOCKED,
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

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let direction = horizontal as Scalar;

    if direction != 0.0 {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
) {
    for gamepad in gamepads.iter() {
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };

        if let Some(x) = axes.get(axis_lx) {
            movement_event_writer.send(MovementAction::Move(x as Scalar));
        }

        let jump_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South,
        };

        if buttons.just_pressed(jump_button) {
            movement_event_writer.send(MovementAction::Jump);
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeCastShape, &Position, &mut Grounded),
        With<CharacterController>,
    >,
    spatial_query: SpatialQuery,
) {
    // Create shape caster as a slightly smaller version of collider

    for (entity, caster_shape, position, mut grounded) in &mut query {
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        if let Some(_hit) = spatial_query.cast_shape(
            &caster_shape.0, // Shape
            position.0,      // Origin
            0.0,             // Shape rotation
            Dir2::NEG_Y,     // Direction
            20.0,            // Maximum time of impact (travel distance)
            true,            // Should initial penetration at the origin be ignored
            filter.clone(),  // Query filter
        ) {
            println!("Ground detected!");
            *grounded = Grounded::Ground;
            continue;
        }

        if let Some(_hit) = spatial_query.cast_shape(
            &caster_shape.0, // Shape
            position.0,      // Origin
            0.0,             // Shape rotation
            Dir2::X,         // Direction
            20.0,            // Maximum time of impact (travel distance)
            true,            // Should initial penetration at the origin be ignored
            filter.clone(),  // Query filter
        ) {
            println!("right wall detected!");
            *grounded = Grounded::RightWall;
            continue;
        }

        if let Some(_hit) = spatial_query.cast_shape(
            &caster_shape.0, // Shape
            position.0,      // Origin
            0.0,             // Shape rotation
            Dir2::NEG_X,     // Direction
            20.0,            // Maximum time of impact (travel distance)
            true,            // Should initial penetration at the origin be ignored
            filter,          // Query filter
        ) {
            println!("left wall detected!");
            *grounded = Grounded::LeftWall;
            continue;
        }

        println!("none detected!");
        *grounded = Grounded::None;
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
        &Grounded,
        &mut CharacterController,
    )>,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for event in movement_event_reader.read() {
        for (movement_acceleration, jump_impulse, mut linear_velocity, grounded, mut controller) in
            &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    linear_velocity.x += *direction * movement_acceleration.0 * delta_time;
                }
                MovementAction::Jump => match *grounded {
                    Grounded::Ground => {
                        linear_velocity.y = jump_impulse.0;
                        controller.wall_jumps = 0;
                        controller.last_wall = None;
                    }
                    Grounded::LeftWall | Grounded::RightWall => {
                        if controller.wall_jumps == 0
                            || controller.last_wall != Some(grounded.clone())
                        {
                            linear_velocity.y = jump_impulse.0;
                            linear_velocity.x = if *grounded == Grounded::LeftWall {
                                jump_impulse.0 * 0.5
                            } else {
                                -jump_impulse.0 * 0.5
                            };
                            controller.wall_jumps += 1;
                            controller.last_wall = Some(grounded.clone());
                        }
                    }
                    Grounded::None => {}
                },
            }
        }
    }
}

/// Slows down movement in the X direction.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
    }
}
