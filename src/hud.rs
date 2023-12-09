use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use crate::{
    damage::DamageEvent,
    player::{Player, PlayerCamera},
    GlobalState,
};

const CROSSHAIR_COLOR: Color = Color::WHITE;
const CROSSHAIR_SIZE: Vec2 = Vec2::new(10.0, 2.0);
const CROSSHAIR_ROTATION: f32 = std::f32::consts::FRAC_PI_4;

const DAMAGE_COLOR: Color = Color::CRIMSON;
const DAMAGE_SIZE: Vec2 = Vec2::new(3.0, 3.0);
const DAMAGE_NUM_OFFSET: u32 = 10;
const DAMAGE_NUM: u32 = 20;
const DAMAGE_DISTANCE: f32 = 2.0;
const DAMAGE_DISPAWN_TIME_SECONDS: f32 = 1.0;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            init_hud,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::MainMenu,
                to: GlobalState::InGame,
            },
            (enable_hud, despawn_all_damage_points),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::Paused,
            },
            disable_hud,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::Paused,
                to: GlobalState::InGame,
            },
            enable_hud,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::GameOver,
            },
            (disable_hud, despawn_all_damage_points),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::InGame,
            },
            (enable_hud, despawn_all_damage_points),
        );

        app.add_systems(
            Update,
            (display_incomming_damage, despawn_damage_points).run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(Component)]
struct HudCamera;

#[derive(Component)]
struct HudDamagePoint {
    spawn_time: f32,
}

fn init_hud(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                is_active: false,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        HudCamera,
    ));

    // Crosshair
    // Top right
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: CROSSHAIR_COLOR,
            custom_size: Some(CROSSHAIR_SIZE),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(10.0, 10.0, 0.0))
            .with_rotation(Quat::from_rotation_z(CROSSHAIR_ROTATION)),
        ..default()
    });

    // Bottom right
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: CROSSHAIR_COLOR,
            custom_size: Some(CROSSHAIR_SIZE),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(10.0, -10.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-CROSSHAIR_ROTATION)),
        ..default()
    });

    // Top left
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: CROSSHAIR_COLOR,
            custom_size: Some(CROSSHAIR_SIZE),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-10.0, 10.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-CROSSHAIR_ROTATION)),
        ..default()
    });

    // Bottom left
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: CROSSHAIR_COLOR,
            custom_size: Some(CROSSHAIR_SIZE),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-10.0, -10.0, 0.0))
            .with_rotation(Quat::from_rotation_z(CROSSHAIR_ROTATION)),
        ..default()
    });
}

fn disable_hud(mut hud_camera: Query<&mut Camera, With<HudCamera>>) {
    let Ok(mut camera) = hud_camera.get_single_mut() else {
        return;
    };

    camera.is_active = false;
}

fn enable_hud(mut hud_camera: Query<&mut Camera, With<HudCamera>>) {
    let Ok(mut camera) = hud_camera.get_single_mut() else {
        return;
    };

    camera.is_active = true;
}

fn display_incomming_damage(
    time: Res<Time>,
    player: Query<Entity, With<Player>>,
    player_camera: Query<&Transform, With<PlayerCamera>>,
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    let Ok(player_camera_transform) = player_camera.get_single() else {
        return;
    };

    for event in damage_events.read() {
        if event.entity != player {
            continue;
        }

        let local_direction = player_camera_transform.rotation.inverse() * event.direction;

        let mut direction = local_direction.xz().extend(0.0);
        direction.x *= -1.0;

        commands
            .spawn((
                TransformBundle::default(),
                InheritedVisibility::VISIBLE,
                HudDamagePoint {
                    spawn_time: time.elapsed_seconds(),
                },
            ))
            .with_children(|builder| {
                for i in DAMAGE_NUM_OFFSET..DAMAGE_NUM_OFFSET + DAMAGE_NUM {
                    let transform =
                        Transform::from_translation(direction * i as f32 * DAMAGE_DISTANCE);
                    builder.spawn((SpriteBundle {
                        sprite: Sprite {
                            color: DAMAGE_COLOR,
                            custom_size: Some(DAMAGE_SIZE),
                            ..default()
                        },
                        transform,
                        ..default()
                    },));
                }
            });
    }
}

fn despawn_damage_points(
    time: Res<Time>,
    points: Query<(Entity, &HudDamagePoint)>,
    mut commands: Commands,
) {
    for (point_entity, point) in points.iter() {
        if point.spawn_time + DAMAGE_DISPAWN_TIME_SECONDS < time.elapsed_seconds() {
            let Some(e) = commands.get_entity(point_entity) else {
                continue;
            };
            e.despawn_recursive();
        }
    }
}

fn despawn_all_damage_points(points: Query<Entity, With<HudDamagePoint>>, mut commands: Commands) {
    for point_entity in points.iter() {
        let Some(e) = commands.get_entity(point_entity) else {
            continue;
        };
        e.despawn_recursive();
    }
}
