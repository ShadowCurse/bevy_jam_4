use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        app.add_systems(Update, player_update);
    }
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
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
                movement_speed: 10.0,
                rotation_speed: 5.0,
            },
        ))
        .with_children(|builder| {
            builder.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 2.0)
                    .looking_at(Vec3::new(1.0, 0.0, 2.0), Vec3::Z),
                ..default()
            });
        });
}

fn player_update(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut controllers: Query<(&Player, &mut Transform, &mut KinematicCharacterController)>,
) {
    let Ok((player, mut transform, mut controller)) = controllers.get_single_mut() else {
        return;
    };

    let rotation: f32 = ev_motion.read().map(|e| -e.delta.x).sum();
    transform.rotate_z(rotation * time.delta_seconds() * player.rotation_speed);

    let mut movement = Vec3::ZERO;
    if keys.pressed(KeyCode::W) {
        movement.x = 1.0;
    }
    if keys.pressed(KeyCode::S) {
        movement.x = -1.0;
    }
    if keys.pressed(KeyCode::A) {
        movement.y = 1.0;
    }
    if keys.pressed(KeyCode::D) {
        movement.y = -1.0;
    }

    let movement = transform.rotation.mul_vec3(movement.normalize())
        * player.movement_speed
        * time.delta_seconds();
    controller.translation = Some(movement);
}
