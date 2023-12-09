use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use crate::GlobalState;

const CROSSHAIR_COLOR: Color = Color::WHITE;
const CROSSHAIR_SIZE: Vec2 = Vec2::new(10.0, 2.0);
const CROSSHAIR_ROTATION: f32 = std::f32::consts::FRAC_PI_4;

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
            enable_hud,
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
            disable_hud,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::InGame,
            },
            enable_hud,
        );
    }
}

#[derive(Component)]
struct HudCamera;

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
