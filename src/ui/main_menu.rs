use bevy::{app::AppExit, prelude::*};

use crate::{GlobalState, UiState, CREATED_BY, GAME_NAME};

use super::{spawn_button, ButtonText, UiConfig};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::MainMenu), setup_main_menu);
        app.add_systems(Update, button_system.run_if(in_state(UiState::MainMenu)));
        app.add_systems(OnExit(UiState::MainMenu), delete_main_menu);
    }
}

#[derive(Component)]
struct MainMenu;

#[derive(Debug, Clone, Copy, Component)]
enum MainMenuButton {
    Play,
    Options,
    Quit,
}

fn setup_main_menu(mut commands: Commands, config: Res<UiConfig>) {
    commands
        .spawn((
            NodeBundle {
                style: config.main_menu_style.clone(),
                background_color: config.panels_background.into(),
                ..default()
            },
            MainMenu,
        ))
        .with_children(|builder| {
            // Title
            builder.spawn(
                (TextBundle {
                    text: Text::from_section(GAME_NAME, config.title_text_style.clone()),
                    ..default()
                })
                .with_style(config.title_style.clone()),
            );

            // Buttons
            builder
                .spawn((NodeBundle {
                    style: config.main_menu_buttons_area_style.clone(),
                    background_color: config.panels_background.into(),
                    ..default()
                },))
                .with_children(|builder| {
                    spawn_button(builder, &config, MainMenuButton::Play);
                    spawn_button(builder, &config, MainMenuButton::Options);
                    spawn_button(builder, &config, MainMenuButton::Quit);
                });

            // Creator
            builder.spawn(
                (TextBundle {
                    text: Text::from_section(CREATED_BY, config.created_by_text_style.clone()),
                    ..default()
                })
                .with_style(config.created_by_style.clone()),
            );
        });
}

#[allow(clippy::complexity)]
fn button_system(
    config: Res<UiConfig>,
    interaction_query: Query<
        (&MainMenuButton, &Interaction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut main_menu_texts: Query<&mut Text, With<ButtonText<MainMenuButton>>>,
    mut main_menu_state: ResMut<NextState<UiState>>,
    mut global_state: ResMut<NextState<GlobalState>>,
    mut exit: EventWriter<AppExit>,
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
                    MainMenuButton::Play => {
                        global_state.set(GlobalState::InGame);
                    }
                    MainMenuButton::Options => {
                        main_menu_state.set(UiState::Options);
                    }
                    MainMenuButton::Quit => exit.send(AppExit),
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

fn delete_main_menu(main_menu: Query<Entity, With<MainMenu>>, mut commands: Commands) {
    let Ok(main_menu) = main_menu.get_single() else {
        return;
    };
    println!("deleting main menu");

    commands.get_entity(main_menu).unwrap().despawn_recursive();
}
