use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        app.add_systems(Update, (player_update, player_move, player_camera_update));
    }
}

#[derive(Component)]
struct Player {
    acceleration: f32,
    slow_down_rade: f32,
    max_movement_speed_squared: f32,
}

#[derive(Component)]
struct PlayerVelocity {
    velocity: Vec3,
}

#[derive(Component)]
struct PlayerCamera {
    rotation_speed: f32,
}

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            TransformBundle::from_transform(Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))),
            RigidBody::KinematicPositionBased,
            Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), 2.0),
            KinematicCharacterController {
                up: Vec3::Z,
                offset: CharacterLength::Relative(0.1),
                ..default()
            },
            Player {
                acceleration: 5.0,
                slow_down_rade: 5.0,
                max_movement_speed_squared: 40.0,
            },
            PlayerVelocity {
                velocity: Vec3::default(),
            },
        ))
        .with_children(|builder| {
            builder.spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 0.0, 2.0)
                        .looking_at(Vec3::new(1.0, 0.0, 2.0), Vec3::Z),
                    ..default()
                },
                PlayerCamera {
                    rotation_speed: 5.0,
                },
            ));
        });
}

fn player_update(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    player_camera_components: Query<&Transform, With<PlayerCamera>>,
    mut player_components: Query<(&Player, &mut PlayerVelocity)>,
) {
    let Ok((player, mut velocity)) = player_components.get_single_mut() else {
        return;
    };

    let Ok(camera_transform) = player_camera_components.get_single() else {
        return;
    };

    // slow down
    let velocity_copy = velocity.velocity;
    velocity.velocity -= velocity_copy * player.slow_down_rade * time.delta_seconds();

    let forward = camera_transform.forward();
    let right = forward.cross(Vec3::Z);

    let mut movement = Vec3::ZERO;
    if keys.pressed(KeyCode::W) {
        movement += forward;
    }
    if keys.pressed(KeyCode::S) {
        movement -= forward;
    }
    if keys.pressed(KeyCode::A) {
        movement -= right;
    }
    if keys.pressed(KeyCode::D) {
        movement += right;
    }

    movement.z = 0.0;
    if movement == Vec3::ZERO {
        return;
    }

    movement = movement.normalize();
    velocity.velocity = movement * player.acceleration * time.delta_seconds();
    let velocity_length = velocity
        .velocity
        .length_squared()
        .max(player.max_movement_speed_squared);
    velocity.velocity = velocity.velocity.normalize() * velocity_length;
}

fn player_move(
    time: Res<Time>,
    mut player_components: Query<
        (&PlayerVelocity, &mut KinematicCharacterController),
        With<Player>,
    >,
) {
    let Ok((velocity, mut controller)) = player_components.get_single_mut() else {
        return;
    };

    let movement = velocity.velocity * time.delta_seconds();
    controller.translation = Some(movement);
}

fn player_camera_update(
    time: Res<Time>,
    mut ev_motion: EventReader<MouseMotion>,
    mut player_camera_components: Query<(&PlayerCamera, &mut Transform)>,
) {
    let Ok((camera, mut transform)) = player_camera_components.get_single_mut() else {
        return;
    };

    let rotation: f32 = ev_motion.read().map(|e| -e.delta.x).sum();
    transform.rotate_z(rotation * time.delta_seconds() * camera.rotation_speed);
}
