use bevy::prelude::*;

use crate::{utils::remove_all_with, GlobalState, UiState};

use super::{spawn_button, ButtonText, UiConfig};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::Paused), setup_pause_menu);
        app.add_systems(Update, button_system.run_if(in_state(UiState::Paused)));
        app.add_systems(OnExit(UiState::Paused), remove_all_with::<PauseMenu>);
    }
}

#[derive(Component)]
struct PauseMenu;

#[derive(Debug, Clone, Copy, Component)]
enum PauseMenuButton {
    Continue,
    Options,
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
            PauseMenu,
        ))
        .with_children(|builder| {
            // Buttons
            builder
                .spawn((NodeBundle {
                    style: config.menu_buttons_area_style.clone(),
                    background_color: config.panels_background.into(),
                    ..default()
                },))
                .with_children(|builder| {
                    spawn_button(builder, &config, PauseMenuButton::Continue);
                    spawn_button(builder, &config, PauseMenuButton::Options);
                    spawn_button(builder, &config, PauseMenuButton::MainMenu);
                });
        });
}

#[allow(clippy::complexity)]
fn button_system(
    config: Res<UiConfig>,
    interaction_query: Query<
        (&PauseMenuButton, &Interaction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut main_menu_texts: Query<&mut Text, With<ButtonText<PauseMenuButton>>>,
    mut main_menu_state: ResMut<NextState<UiState>>,
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
                    PauseMenuButton::Continue => {
                        global_state.set(GlobalState::InGame);
                    }
                    PauseMenuButton::Options => {
                        main_menu_state.set(UiState::Options);
                    }
                    PauseMenuButton::MainMenu => {
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
