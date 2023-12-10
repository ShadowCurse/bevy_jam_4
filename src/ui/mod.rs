use std::marker::PhantomData;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::CursorGrabMode,
};
use bevy_asset_loader::prelude::*;

use crate::{utils::set_state, GlobalState, UiState};

mod game_over;
mod game_won;
mod main_menu;
mod options;
mod pause;
mod stats;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, UiAssets>(GlobalState::AssetLoading);

        app.add_plugins(game_over::GameOverPlugin);
        app.add_plugins(game_won::GameWonPlugin);
        app.add_plugins(stats::StatsPlugin);
        app.add_plugins(main_menu::MainMenuPlugin);
        app.add_plugins(options::OptionsPlugin);
        app.add_plugins(pause::PausePlugin);

        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            (
                init_resources,
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
            (set_state::<UiState, { UiState::Stats as u8 }>, grab_mouse),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::Paused,
            },
            (
                set_state::<UiState, { UiState::Paused as u8 }>,
                release_mouse,
            ),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::Paused,
                to: GlobalState::InGame,
            },
            (set_state::<UiState, { UiState::Stats as u8 }>, grab_mouse),
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
            (
                set_state::<UiState, { UiState::GameOver as u8 }>,
                release_mouse,
            ),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::InGame,
            },
            (set_state::<UiState, { UiState::Stats as u8 }>, grab_mouse),
        );
        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::MainMenu,
            },
            set_state::<UiState, { UiState::MainMenu as u8 }>,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::GameWon,
            },
            (
                set_state::<UiState, { UiState::GameWon as u8 }>,
                release_mouse,
            ),
        );
        app.add_systems(
            OnTransition {
                from: GlobalState::GameWon,
                to: GlobalState::MainMenu,
            },
            set_state::<UiState, { UiState::MainMenu as u8 }>,
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

    pub stats_style: Style,
    pub stats_columns_style: Style,
    pub stats_big_text_style: TextStyle,
    pub stats_normal_text_style: TextStyle,
}

#[derive(Resource)]
pub struct UiResources {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct ButtonText<T> {
    _phatom: PhantomData<T>,
}

fn init_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = Extent3d {
        width: 1280,
        height: 720,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        emissive: Color::WHITE,
        perceptual_roughness: 0.9,
        // reflectance: 0.02,
        unlit: true,
        ..default()
    });
    let aspect_ration = size.width as f32 / size.height as f32;
    let mesh_width = 0.5;
    let mesh_hight = mesh_width / aspect_ration;
    let mesh_size = Vec2::new(mesh_width, mesh_hight);
    let mesh_handle = meshes.add(shape::Quad::new(mesh_size).into());

    let first_pass_layer = RenderLayers::layer(1);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(image_handle),
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
        first_pass_layer,
    ));

    commands.insert_resource(UiResources {
        mesh: mesh_handle,
        material: material_handle,
    })
}

fn setup_ui_config(ui_assets: Res<UiAssets>, mut commands: Commands) {
    commands.insert_resource(UiConfig {
        clear_background: Color::NONE,
        panels_background: Color::NONE, //BLACK,
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
            font_size: 30.0,
            color: Color::WHITE,
        },

        menu_style: Style {
            display: Display::Grid,
            margin: UiRect::all(Val::Auto),
            justify_items: JustifyItems::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
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
            justify_items: JustifyItems::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
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

        stats_style: Style {
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            column_gap: Val::Percent(30.0),
            ..default()
        },
        stats_columns_style: Style {
            display: Display::Grid,
            justify_self: JustifySelf::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        stats_big_text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 150.0,
            color: Color::WHITE,
        },
        stats_normal_text_style: TextStyle {
            font: ui_assets.font.clone(),
            font_size: 100.0,
            color: Color::WHITE,
        },
    });
}

fn grab_mouse(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn release_mouse(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = true;
    window.cursor.grab_mode = CursorGrabMode::None;
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
