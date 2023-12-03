use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::weapons::{FreeFloatingWeapon, ShootEvent, Weapon};

const PLAYER_WEAPON_DEFAULT_TRANSLATION: Vec3 = Vec3::new(0.0, -0.5, -1.4);
const PLAYER_THROW_OFFSET_SCALE: f32 = 10.0;
const PLAYER_THROW_STRENGTH: f32 = 20.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        app.add_systems(
            Update,
            (
                player_shoot,
                player_pick_up_weapon,
                player_throw_weapon,
                player_update,
                player_move,
                player_camera_update,
                player_weapon_update,
            ),
        );
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
    was_input: bool,
    velocity: Vec3,
}

#[derive(Component)]
struct PlayerCamera {
    default_translation: Vec3,
    rotation_speed: f32,

    bounce_continue: bool,
    bounce_progress: f32,
    bounce_speed: f32,

    bounce_amplitude: f32,
    bounce_amplitude_modifier: f32,
    bounce_amplitude_modifier_speed: f32,
    bounce_amplitude_modifier_max: f32,
}

#[derive(Component)]
struct PlayerWeapon {
    default_translation: Vec3,

    bounce_continue: bool,
    bounce_progress: f32,
    bounce_speed: f32,
    bounce_amplitude: f32,
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
                was_input: false,
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
                // TransformBundle::from_transform(
                //     Transform::from_xyz(0.0, 0.0, 2.0) //.with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                //         .looking_at(Vec3::new(1.0, 0.0, 2.0), Vec3::Z),
                // ),
                PlayerCamera {
                    default_translation: Vec3::new(0.0, 0.0, 2.0),
                    rotation_speed: 5.0,

                    bounce_continue: false,
                    bounce_progress: 0.0,
                    bounce_speed: 8.0,

                    bounce_amplitude: 0.2,
                    bounce_amplitude_modifier: 1.0,
                    bounce_amplitude_modifier_speed: 1.0,
                    bounce_amplitude_modifier_max: 2.0,
                },
                Collider::ball(0.5),
            ));
        });
}

fn player_pick_up_weapon(
    rapier_context: Res<RapierContext>,
    player: Query<Entity, With<Player>>,
    player_camera: Query<Entity, With<PlayerCamera>>,
    weapons: Query<Entity, With<Weapon>>,
    mut commands: Commands,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    let Ok(camera) = player_camera.get_single() else {
        return;
    };

    for weapon in weapons.iter() {
        for contact_pair in rapier_context.contacts_with(weapon) {
            if contact_pair.collider1() == player || contact_pair.collider2() == player {
                commands.get_entity(camera).unwrap().add_child(weapon);
                commands
                    .get_entity(weapon)
                    .unwrap()
                    .insert(PlayerWeapon {
                        default_translation: PLAYER_WEAPON_DEFAULT_TRANSLATION,
                        bounce_continue: false,
                        bounce_progress: 0.0,
                        bounce_speed: 4.0,
                        bounce_amplitude: 0.08,
                    })
                    .remove::<(Collider, FreeFloatingWeapon)>();
            }
        }
    }
}

fn player_throw_weapon(
    keys: Res<Input<KeyCode>>,
    player_camera: Query<Entity, With<PlayerCamera>>,
    player_weapon_components: Query<(Entity, &GlobalTransform), With<PlayerWeapon>>,
    mut commands: Commands,
) {
    let Ok(camera) = player_camera.get_single() else {
        return;
    };

    let Ok((weapon, weapon_global_transform)) = player_weapon_components.get_single() else {
        return;
    };

    if keys.pressed(KeyCode::F) {
        commands
            .get_entity(camera)
            .unwrap()
            .remove_children(&[weapon]);

        commands
            .get_entity(weapon)
            .unwrap()
            .remove::<PlayerWeapon>()
            .insert((
                Transform::from_translation(
                    weapon_global_transform.translation()
                        + weapon_global_transform.forward() * PLAYER_THROW_OFFSET_SCALE,
                )
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                Collider::cuboid(0.6, 0.6, 0.3),
                RigidBody::Dynamic,
                Velocity {
                    linvel: weapon_global_transform.forward() * PLAYER_THROW_STRENGTH,
                    ..default()
                },
            ));
    }
}

fn player_shoot(
    keys: Res<Input<KeyCode>>,
    player_weapon_components: Query<(Entity, &GlobalTransform, &Weapon), With<PlayerWeapon>>,
    mut shoot_event: EventWriter<ShootEvent>,
) {
    let Ok((weapon_entity, weapon_global_transform, weapon_attack_timer)) =
        player_weapon_components.get_single()
    else {
        return;
    };

    if keys.pressed(KeyCode::Space) && weapon_attack_timer.attack_timer.finished() {
        shoot_event.send(ShootEvent {
            weapon_entity,
            weapon_translation: weapon_global_transform.translation(),
            direction: weapon_global_transform.forward(),
        });
    }
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
        velocity.was_input = false;
        return;
    }

    movement = movement.normalize();
    velocity.velocity = movement * player.acceleration * time.delta_seconds();
    let velocity_length = velocity
        .velocity
        .length_squared()
        .max(player.max_movement_speed_squared);
    velocity.velocity = velocity.velocity.normalize() * velocity_length;
    velocity.was_input = true;
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

// TODO make better
fn player_camera_update(
    time: Res<Time>,
    player_components: Query<&PlayerVelocity>,
    mut ev_motion: EventReader<MouseMotion>,
    mut player_camera_components: Query<(&mut PlayerCamera, &mut Transform)>,
) {
    let Ok(velocity) = player_components.get_single() else {
        return;
    };

    let Ok((mut camera, mut transform)) = player_camera_components.get_single_mut() else {
        return;
    };

    let rotation: f32 = ev_motion.read().map(|e| -e.delta.x).sum();
    transform.rotate_z(rotation * time.delta_seconds() * camera.rotation_speed);

    transform.translation = camera.default_translation
        + Vec3::NEG_Z
            * camera.bounce_amplitude
            * camera.bounce_amplitude_modifier
            * (camera.bounce_progress).sin();

    if velocity.was_input {
        // if there was input, continue bouncing
        camera.bounce_continue = true;
        camera.bounce_progress += camera.bounce_speed * time.delta_seconds();
        camera.bounce_amplitude_modifier = (camera.bounce_amplitude_modifier
            + camera.bounce_amplitude_modifier_speed * time.delta_seconds())
        .min(camera.bounce_amplitude_modifier_max);
    } else if camera.bounce_continue {
        // if there was no input, continue until next PI
        camera.bounce_progress += camera.bounce_speed * time.delta_seconds();
        let next_pi = (camera.bounce_progress / std::f32::consts::PI).ceil() * std::f32::consts::PI;
        if next_pi <= camera.bounce_progress + 0.1 {
            camera.bounce_progress = 0.0;
            camera.bounce_continue = false;
            camera.bounce_amplitude_modifier = 1.0;
        }
    }
}

// TODO make better
fn player_weapon_update(
    time: Res<Time>,
    player_velocity: Query<&PlayerVelocity>,
    mut weapon: Query<(&mut Transform, &mut PlayerWeapon)>,
) {
    let Ok(velocity) = player_velocity.get_single() else {
        return;
    };

    let Ok((mut weapon_transform, mut player_weapon)) = weapon.get_single_mut() else {
        return;
    };
    weapon_transform.rotation = Quat::IDENTITY;

    let bounce = player_weapon.bounce_progress.sin();
    let offset = Vec3::new(
        player_weapon.bounce_amplitude * bounce,
        (player_weapon.bounce_amplitude * bounce).abs(),
        0.0,
    );

    weapon_transform.translation = player_weapon.default_translation + offset;

    if velocity.was_input {
        // if there was input, continue bouncing
        player_weapon.bounce_continue = true;
        player_weapon.bounce_progress += player_weapon.bounce_speed * time.delta_seconds();
    } else if player_weapon.bounce_continue {
        // if there was no input, continue until next PI
        player_weapon.bounce_progress += player_weapon.bounce_speed * time.delta_seconds();
        let next_pi =
            (player_weapon.bounce_progress / std::f32::consts::PI).ceil() * std::f32::consts::PI;
        if next_pi <= player_weapon.bounce_progress + 0.1 {
            player_weapon.bounce_progress = 0.0;
            player_weapon.bounce_continue = false;
        }
    }
}
