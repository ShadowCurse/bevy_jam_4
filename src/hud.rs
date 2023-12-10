use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use crate::{
    damage::DamageEvent,
    level::{LevelInfo, LevelStarted},
    player::{Player, PlayerCamera},
    ui::UiAssets,
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

const TUTORIAL_TEXT: &str =
    "WASD - Move\nSPACE - Shoot\nF - throw a weapon\n(Throwing weapons also deal damage)";
const TUTORIAL_TEXT_DISPAWN_TIME_SECONDS: f32 = 5.0;
const BOSS_TEXT: &str = "THE RED DRAGON LAIR";
const BOSS_TEXT_DISPAWN_TIME_SECONDS: f32 = 2.0;

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
            (enable_hud, despawn_all_timed_elements, show_tutorial_text).chain(),
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
            (disable_hud, despawn_all_timed_elements),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::InGame,
            },
            (enable_hud, despawn_all_timed_elements),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::GameWon,
            },
            (disable_hud, despawn_all_timed_elements),
        );

        app.add_systems(
            Update,
            (
                display_incomming_damage,
                progress_timed_elements,
                show_boss_text,
            )
                .run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(Component)]
struct HudCamera;

#[derive(Component)]
struct HudTimedElement {
    spawn_time: f32,
    lifespawn: f32,
}

#[derive(Resource)]
struct HudResources {
    text_style: TextStyle,
    boss_text_style: TextStyle,
}

fn init_hud(ui_assets: Res<UiAssets>, mut commands: Commands) {
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

    commands.insert_resource(HudResources {
        text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 60.0,
            color: Color::WHITE,
        },
        boss_text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 80.0,
            color: Color::ORANGE_RED,
        },
    })
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

fn show_tutorial_text(time: Res<Time>, hud_resources: Res<HudResources>, mut commands: Commands) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(TUTORIAL_TEXT, hud_resources.text_style.clone())
                .with_alignment(TextAlignment::Center),
            ..default()
        },
        HudTimedElement {
            spawn_time: time.elapsed_seconds(),
            lifespawn: TUTORIAL_TEXT_DISPAWN_TIME_SECONDS,
        },
    ));
}

fn show_boss_text(
    time: Res<Time>,
    hud_resources: Res<HudResources>,
    level_info: Res<LevelInfo>,
    mut commands: Commands,
    mut level_started_events: EventReader<LevelStarted>,
) {
    for _ in level_started_events.read() {
        if level_info.game_progress == 100 {
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(BOSS_TEXT, hud_resources.boss_text_style.clone())
                        .with_alignment(TextAlignment::Center),
                    ..default()
                },
                HudTimedElement {
                    spawn_time: time.elapsed_seconds(),
                    lifespawn: BOSS_TEXT_DISPAWN_TIME_SECONDS,
                },
            ));
        }
    }
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
                HudTimedElement {
                    spawn_time: time.elapsed_seconds(),
                    lifespawn: DAMAGE_DISPAWN_TIME_SECONDS,
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

fn progress_timed_elements(
    time: Res<Time>,
    points: Query<(Entity, &HudTimedElement)>,
    mut commands: Commands,
) {
    for (point_entity, point) in points.iter() {
        if point.spawn_time + point.lifespawn < time.elapsed_seconds() {
            let Some(e) = commands.get_entity(point_entity) else {
                continue;
            };
            e.despawn_recursive();
        }
    }
}

fn despawn_all_timed_elements(
    points: Query<Entity, With<HudTimedElement>>,
    mut commands: Commands,
) {
    for point_entity in points.iter() {
        let Some(e) = commands.get_entity(point_entity) else {
            continue;
        };
        e.despawn_recursive();
    }
}
