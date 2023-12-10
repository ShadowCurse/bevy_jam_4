use bevy::prelude::*;

use crate::{
    damage::Health,
    level::LevelInfo,
    player::{Player, PlayerWeapon},
    utils::remove_all_with,
    weapons::Ammo,
    UiState,
};

use super::UiConfig;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::Stats), setup_stats_menu);
        app.add_systems(
            Update,
            (update_plyaer_hp, update_player_ammo, update_game_progress)
                .run_if(in_state(UiState::Stats)),
        );
        app.add_systems(OnExit(UiState::Stats), remove_all_with::<StatsMenu>);
    }
}

#[derive(Component)]
struct StatsMenu;

#[derive(Component)]
struct StatsGameProgress;

#[derive(Component)]
struct StatsPlayerHp;

#[derive(Component)]
struct StatsPlayerAmmo;

fn setup_stats_menu(mut commands: Commands, config: Res<UiConfig>) {
    commands
        .spawn((
            NodeBundle {
                style: config.stats_style.clone(),
                background_color: config.panels_background.into(),
                ..default()
            },
            StatsMenu,
        ))
        .with_children(|builder| {
            // Left column (Ammo + HP)
            builder
                .spawn((
                    NodeBundle {
                        style: config.stats_columns_style.clone(),
                        background_color: config.panels_background.into(),
                        ..default()
                    },
                    StatsMenu,
                ))
                .with_children(|builder| {
                    // Ammo
                    builder.spawn((TextBundle {
                        text: Text::from_section("AMMO", config.stats_normal_text_style.clone()),
                        ..default()
                    }
                    .with_style(config.title_style.clone()),));
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section("", config.stats_normal_text_style.clone()),
                            ..default()
                        }
                        .with_style(config.title_style.clone()),
                        StatsPlayerAmmo,
                    ));

                    // HP
                    builder.spawn((TextBundle {
                        text: Text::from_section("HP", config.stats_normal_text_style.clone()),
                        ..default()
                    }
                    .with_style(config.title_style.clone()),));
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section("", config.stats_normal_text_style.clone()),
                            ..default()
                        }
                        .with_style(config.title_style.clone()),
                        StatsPlayerHp,
                    ));
                });

            // Right column (Score)
            builder
                .spawn((
                    NodeBundle {
                        style: config.stats_columns_style.clone(),
                        background_color: config.panels_background.into(),
                        ..default()
                    },
                    StatsMenu,
                ))
                .with_children(|builder| {
                    // "Score" text
                    builder.spawn((TextBundle {
                        text: Text::from_section("PROGRESS", config.stats_big_text_style.clone()),
                        ..default()
                    }
                    .with_style(config.title_style.clone()),));

                    // Actual score number
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section("", config.stats_big_text_style.clone()),
                            ..default()
                        }
                        .with_style(config.title_style.clone()),
                        StatsGameProgress,
                    ));
                });
        });
}

fn update_player_ammo(
    player_ammo: Query<&Ammo, With<PlayerWeapon>>,
    mut window_mode_text: Query<&mut Text, With<StatsPlayerAmmo>>,
) {
    let mut text = window_mode_text.single_mut();
    match player_ammo.get_single() {
        Ok(ammo) => text.sections[0].value = format!("{}", ammo.ammo),
        Err(_) => text.sections[0].value = format!("---"),
    }
}

fn update_plyaer_hp(
    player_hp: Query<&Health, With<Player>>,
    mut volume_text: Query<&mut Text, With<StatsPlayerHp>>,
) {
    let Ok(hp) = player_hp.get_single() else {
        return;
    };
    let mut text = volume_text.single_mut();
    text.sections[0].value = format!("{}", hp.health);
}

fn update_game_progress(
    level_info: Res<LevelInfo>,
    mut volume_text: Query<&mut Text, With<StatsGameProgress>>,
) {
    let mut text = volume_text.single_mut();
    text.sections[0].value = format!("{}%", level_info.game_progress);
}
