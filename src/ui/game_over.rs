use bevy::prelude::*;

use crate::{utils::remove_all_with, GlobalState, UiState};

use super::{spawn_button, ButtonText, UiConfig};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::GameOver), setup_pause_menu);
        app.add_systems(Update, button_system.run_if(in_state(UiState::GameOver)));
        app.add_systems(OnExit(UiState::GameOver), remove_all_with::<GameOverMenu>);
    }
}

#[derive(Component)]
struct GameOverMenu;

#[derive(Debug, Clone, Copy, Component)]
enum GameOverMenuButton {
    Restart,
    MainMenu,
}

fn setup_pause_menu(mut commands: Commands, config: Res<UiConfig>) {
    commands
        .spawn((
            NodeBundle {
                style: config.menu_style.clone(),
                background_color: config.panels_background.into(),
                ..default()
            },
            GameOverMenu,
        ))
        .with_children(|builder| {
            builder.spawn(
                (TextBundle {
                    text: Text::from_section("YOU DIED", config.title_text_style.clone()),
                    ..default()
                })
                .with_style(config.title_style.clone()),
            );

            // Buttons
            builder
                .spawn((NodeBundle {
                    style: config.menu_buttons_area_style.clone(),
                    background_color: config.panels_background.into(),
                    ..default()
                },))
                .with_children(|builder| {
                    spawn_button(builder, &config, GameOverMenuButton::Restart);
                    spawn_button(builder, &config, GameOverMenuButton::MainMenu);
                });
        });
}

#[allow(clippy::complexity)]
fn button_system(
    config: Res<UiConfig>,
    interaction_query: Query<
        (&GameOverMenuButton, &Interaction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut main_menu_texts: Query<&mut Text, With<ButtonText<GameOverMenuButton>>>,
    mut global_state: ResMut<NextState<GlobalState>>,
) {
    for (button, interaction, children) in interaction_query.iter() {
        let text_entity = children[0];
        let Ok(mut text) = main_menu_texts.get_mut(text_entity) else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].style.color = config.button_text_color_pressed;
                match button {
                    GameOverMenuButton::Restart => {
                        global_state.set(GlobalState::InGame);
                    }
                    GameOverMenuButton::MainMenu => {
                        global_state.set(GlobalState::MainMenu);
                    }
                }
            }
            Interaction::Hovered => {
                text.sections[0].style.color = config.button_text_color_hover;
            }
            Interaction::None => {
                text.sections[0].style.color = config.button_text_color_normal;
            }
        }
    }
}
