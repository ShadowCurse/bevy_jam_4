use bevy::prelude::*;

use crate::{damage::Health, player::Player, utils::remove_all_with, UiState};

use super::UiConfig;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::Hud), setup_main_menu);
        app.add_systems(
            Update,
            (update_plyaer_hp, update_player_ammo, update_plyaer_score)
                .run_if(in_state(UiState::Hud)),
        );
        app.add_systems(OnExit(UiState::Hud), remove_all_with::<HudMenu>);
    }
}

#[derive(Component)]
struct HudMenu;

#[derive(Component)]
struct HudPlayerScore;

#[derive(Component)]
struct HudPlayerHp;

#[derive(Component)]
struct HudPlayerAmmo;

fn setup_main_menu(mut commands: Commands, config: Res<UiConfig>) {
    commands
        .spawn((
            NodeBundle {
                style: config.hud_style.clone(),
                background_color: config.panels_background.into(),
                ..default()
            },
            HudMenu,
        ))
        .with_children(|builder| {
            // Left column (Ammo + HP)
            builder
                .spawn((
                    NodeBundle {
                        style: config.hud_columns_style.clone(),
                        background_color: config.panels_background.into(),
                        ..default()
                    },
                    HudMenu,
                ))
                .with_children(|builder| {
                    // Ammo
                    builder.spawn((TextBundle {
                        text: Text::from_section("AMMO", config.hud_normal_text_style.clone()),
                        ..default()
                    }
                    .with_style(config.title_style.clone()),));
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section("", config.hud_normal_text_style.clone()),
                            ..default()
                        }
                        .with_style(config.title_style.clone()),
                        HudPlayerAmmo,
                    ));

                    // HP
                    builder.spawn((TextBundle {
                        text: Text::from_section("HP", config.hud_normal_text_style.clone()),
                        ..default()
                    }
                    .with_style(config.title_style.clone()),));
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section("", config.hud_normal_text_style.clone()),
                            ..default()
                        }
                        .with_style(config.title_style.clone()),
                        HudPlayerHp,
                    ));
                });

            // Right column (Score)
            builder
                .spawn((
                    NodeBundle {
                        style: config.hud_columns_style.clone(),
                        background_color: config.panels_background.into(),
                        ..default()
                    },
                    HudMenu,
                ))
                .with_children(|builder| {
                    // "Score" text
                    builder.spawn((TextBundle {
                        text: Text::from_section("SCORE", config.hud_big_text_style.clone()),
                        ..default()
                    }
                    .with_style(config.title_style.clone()),));

                    // Actual score number
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section("", config.hud_big_text_style.clone()),
                            ..default()
                        }
                        .with_style(config.title_style.clone()),
                        HudPlayerScore,
                    ));
                });
        });
}

fn update_player_ammo(mut window_mode_text: Query<&mut Text, With<HudPlayerAmmo>>) {
    let mut text = window_mode_text.single_mut();
    text.sections[0].value = format!("---");
}

fn update_plyaer_hp(
    player_hp: Query<&Health, With<Player>>,
    mut volume_text: Query<&mut Text, With<HudPlayerHp>>,
) {
    let Ok(hp) = player_hp.get_single() else {
        return;
    };
    let mut text = volume_text.single_mut();
    text.sections[0].value = format!("{}", hp.health);
}

fn update_plyaer_score(mut volume_text: Query<&mut Text, With<HudPlayerScore>>) {
    let mut text = volume_text.single_mut();
    text.sections[0].value = format!("{}", 69);
}
