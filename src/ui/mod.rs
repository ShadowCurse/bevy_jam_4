use std::marker::PhantomData;

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{utils::set_state, GlobalState, UiState};

mod main_menu;
mod options;
mod pause;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, UiAssets>(GlobalState::AssetLoading);

        app.add_plugins(main_menu::MainMenuPlugin);
        app.add_plugins(options::OptionsPlugin);
        app.add_plugins(pause::PausePlugin);

        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            (
                spawn_ui_camera,
                setup_ui_config,
                set_state::<UiState, { UiState::MainMenu as u8 }>,
            )
                .chain(),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::MainMenu,
                to: GlobalState::InGame,
            },
            set_state::<UiState, { UiState::Hud as u8 }>,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::Paused,
            },
            set_state::<UiState, { UiState::Paused as u8 }>,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::Paused,
                to: GlobalState::InGame,
            },
            set_state::<UiState, { UiState::Hud as u8 }>,
        );
        app.add_systems(
            OnTransition {
                from: GlobalState::Paused,
                to: GlobalState::MainMenu,
            },
            set_state::<UiState, { UiState::MainMenu as u8 }>,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::GameOver,
            },
            set_state::<UiState, { UiState::GameOver as u8 }>,
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "fonts/monaco.ttf")]
    font: Handle<Font>,
}

#[derive(Debug, Clone, Resource)]
pub struct UiConfig {
    pub clear_background: Color,
    pub panels_background: Color,
    pub button_background: Color,
    pub button_text_color_normal: Color,
    pub button_text_color_hover: Color,
    pub button_text_color_pressed: Color,

    pub button_style: Style,

    pub text_style: TextStyle,

    pub menu_style: Style,
    pub menu_buttons_area_style: Style,

    pub options_text_style: TextStyle,
    pub options_buttons_area_style: Style,

    pub title_style: Style,
    pub title_text_style: TextStyle,

    pub created_by_style: Style,
    pub created_by_text_style: TextStyle,
}

#[derive(Component)]
pub struct ButtonText<T> {
    _phatom: PhantomData<T>,
}

fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });
}

fn setup_ui_config(ui_assets: Res<UiAssets>, mut commands: Commands) {
    commands.insert_resource(UiConfig {
        clear_background: Color::NONE,
        panels_background: Color::BLACK,
        button_background: Color::NONE,

        button_text_color_normal: Color::WHITE,
        button_text_color_hover: Color::ORANGE_RED,
        button_text_color_pressed: Color::RED,

        button_style: Style {
            margin: UiRect::all(Val::Percent(10.0)),
            justify_self: JustifySelf::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 40.0,
            color: Color::WHITE,
        },

        menu_style: Style {
            display: Display::Grid,
            width: Val::Percent(100.0),
            height: Val::Percent(80.0),
            margin: UiRect::all(Val::Auto),
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        menu_buttons_area_style: Style {
            display: Display::Grid,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            ..default()
        },

        options_buttons_area_style: Style {
            display: Display::Grid,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        options_text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 50.0,
            color: Color::ORANGE_RED,
        },

        title_style: Style {
            justify_self: JustifySelf::Center,
            ..default()
        },
        title_text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 90.0,
            color: Color::WHITE,
        },

        created_by_style: Style {
            justify_self: JustifySelf::Center,
            ..default()
        },
        created_by_text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 35.0,
            color: Color::WHITE,
        },
    });
}

fn spawn_button<B>(builder: &mut ChildBuilder, style: &UiConfig, button: B)
where
    B: Component + std::fmt::Debug + Copy,
{
    builder
        .spawn((
            ButtonBundle {
                style: style.button_style.clone(),
                background_color: style.button_background.into(),
                ..default()
            },
            button,
        ))
        .with_children(|builder| {
            builder.spawn((
                TextBundle {
                    text: Text::from_section(format!("{button:?}"), style.text_style.clone()),
                    ..default()
                },
                ButtonText::<B> {
                    _phatom: Default::default(),
                },
            ));
        });
}
