use bevy::{prelude::*, window::WindowMode};

use crate::{utils::remove_all_with, GameSettings, GlobalState, UiState};

use super::{spawn_button, ButtonText, UiConfig};

pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::Options), setup_option_menu);
        app.add_systems(
            Update,
            (
                button_system,
                update_window_mode_text,
                update_volume_value_text,
                update_camera_sense_value_text,
            )
                .run_if(in_state(UiState::Options)),
        );
        app.add_systems(OnExit(UiState::Options), remove_all_with::<OptionsMenu>);
    }
}

#[derive(Component)]
struct OptionsMenu;

#[derive(Debug, Clone, Copy, Component)]
enum OptionMenuButton {
    FullScreen,
    Windowed,
    VolumeUp,
    VolumeDown,
    SenseUp,
    SenseDown,
    Back,
}

#[derive(Component)]
struct OptionsWindowModeText;

#[derive(Component)]
struct OptionsVolumeText;

#[derive(Component)]
struct OptionsCameraSenseText;

fn setup_option_menu(mut commands: Commands, config: Res<UiConfig>) {
    commands
        .spawn((
            NodeBundle {
                style: config.menu_style.clone(),
                background_color: config.panels_background.into(),
                ..default()
            },
            OptionsMenu,
        ))
        .with_children(|builder| {
            // 2 rows of settings
            // and 1 Back button
            builder
                .spawn((NodeBundle {
                    style: config.menu_buttons_area_style.clone(),
                    background_color: config.panels_background.into(),
                    ..default()
                },))
                .with_children(|builder| {
                    // Window mode
                    builder
                        .spawn((NodeBundle {
                            style: config.options_buttons_area_style.clone(),
                            background_color: config.panels_background.into(),
                            ..default()
                        },))
                        .with_children(|builder| {
                            spawn_button(builder, &config, OptionMenuButton::FullScreen);
                            spawn_button(builder, &config, OptionMenuButton::Windowed);
                            // Window mode text
                            builder.spawn((
                                TextBundle {
                                    text: Text::from_section("", config.options_text_style.clone()),
                                    ..default()
                                }
                                .with_style(config.button_style.clone()),
                                OptionsWindowModeText,
                            ));
                        });

                    // Volume
                    builder
                        .spawn((NodeBundle {
                            style: config.options_buttons_area_style.clone(),
                            background_color: config.panels_background.into(),
                            ..default()
                        },))
                        .with_children(|builder| {
                            spawn_button(builder, &config, OptionMenuButton::VolumeUp);
                            spawn_button(builder, &config, OptionMenuButton::VolumeDown);
                            builder.spawn((
                                TextBundle {
                                    text: Text::from_section("", config.options_text_style.clone()),
                                    ..default()
                                }
                                .with_style(config.button_style.clone()),
                                OptionsVolumeText,
                            ));
                        });

                    // Camera sense
                    builder
                        .spawn((NodeBundle {
                            style: config.options_buttons_area_style.clone(),
                            background_color: config.panels_background.into(),
                            ..default()
                        },))
                        .with_children(|builder| {
                            spawn_button(builder, &config, OptionMenuButton::SenseUp);
                            spawn_button(builder, &config, OptionMenuButton::SenseDown);
                            builder.spawn((
                                TextBundle {
                                    text: Text::from_section("", config.options_text_style.clone()),
                                    ..default()
                                }
                                .with_style(config.button_style.clone()),
                                OptionsCameraSenseText,
                            ));
                        });

                    spawn_button(builder, &config, OptionMenuButton::Back);
                });
        });
}

#[allow(clippy::complexity)]
fn button_system(
    config: Res<UiConfig>,
    interaction_query: Query<
        (&OptionMenuButton, &Interaction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    global_state: Res<State<GlobalState>>,
    // audio: ResMut<Audio>,
    mut windows: Query<&mut Window>,
    mut game_settings: ResMut<GameSettings>,
    mut texts: Query<&mut Text, With<ButtonText<OptionMenuButton>>>,
    mut ui_state: ResMut<NextState<UiState>>,
) {
    for (button, interaction, children) in interaction_query.iter() {
        let text_entity = children[0];
        let Ok(mut text) = texts.get_mut(text_entity) else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].style.color = config.button_text_color_pressed;
                match button {
                    OptionMenuButton::Windowed => {
                        game_settings.window_mode = WindowMode::Windowed;
                        windows.single_mut().mode = WindowMode::Windowed;
                    }
                    OptionMenuButton::FullScreen => {
                        game_settings.window_mode = WindowMode::Fullscreen;
                        windows.single_mut().mode = WindowMode::Fullscreen;
                    }
                    OptionMenuButton::VolumeUp => {
                        game_settings.volume += 0.05;
                    }
                    OptionMenuButton::VolumeDown => {
                        game_settings.volume -= 0.05;
                        if game_settings.volume <= 0.0 {
                            game_settings.volume = 0.0;
                        }
                    }
                    OptionMenuButton::SenseUp => {
                        game_settings.camera_sensitivity += 0.5;
                    }
                    OptionMenuButton::SenseDown => {
                        game_settings.camera_sensitivity -= 0.5;
                        if game_settings.camera_sensitivity <= 0.0 {
                            game_settings.camera_sensitivity = 0.0;
                        }
                    }
                    OptionMenuButton::Back => match global_state.get() {
                        GlobalState::MainMenu => ui_state.set(UiState::MainMenu),
                        GlobalState::Paused => ui_state.set(UiState::Paused),
                        _ => {}
                    },
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

fn update_window_mode_text(
    game_settings: Res<GameSettings>,
    mut window_mode_text: Query<&mut Text, With<OptionsWindowModeText>>,
) {
    let mut text = window_mode_text.single_mut();
    text.sections[0].value = format!("{:?}", game_settings.window_mode);
}

fn update_volume_value_text(
    game_settings: Res<GameSettings>,
    mut volume_text: Query<&mut Text, With<OptionsVolumeText>>,
) {
    let mut text = volume_text.single_mut();
    text.sections[0].value = format!("{:.2}", game_settings.volume);
}

fn update_camera_sense_value_text(
    game_settings: Res<GameSettings>,
    mut volume_text: Query<&mut Text, With<OptionsCameraSenseText>>,
) {
    let mut text = volume_text.single_mut();
    text.sections[0].value = format!("{:.2}", game_settings.camera_sensitivity);
}
