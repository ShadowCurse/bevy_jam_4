use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_rapier3d::prelude::*;

mod damage;
mod enemies;
mod hud;
mod level;
mod player;
mod ui;
mod utils;
mod weapons;

use utils::IntoState;

const GAME_NAME: &str = "Fridges must die";
const CREATED_BY: &str = "Created by ShadowCurse";

const COLLISION_GROUP_LEVEL: Group = Group::GROUP_1;
const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
const COLLISION_GROUP_ENEMY: Group = Group::GROUP_3;
const COLLISION_GROUP_PROJECTILES: Group = Group::GROUP_4;
const COLLISION_GROUP_PICKUP: Group = Group::GROUP_5;

fn main() {
    let mut app = App::new();

    app.add_state::<GlobalState>();
    app.add_state::<UiState>();

    app.add_loading_state(
        LoadingState::new(GlobalState::AssetLoading).continue_to_state(GlobalState::MainMenu),
    );

    app.insert_resource(AssetMetaCheck::Never);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_NAME.to_string(),
                mode: WindowMode::Windowed,
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }),
        RapierPhysicsPlugin::<NoUserData>::default(),
        // RapierDebugRenderPlugin::default(),
        AudioPlugin,
        damage::DamagePlugin,
        enemies::EnemiesPlugin,
        hud::HudPlugin,
        level::LevelPlugin,
        ui::UiPlugin,
        player::PlayerPlugin,
        weapons::WeaponsPlugin,
    ));

    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });
    app.insert_resource(ClearColor(Color::BLACK));

    app.insert_resource(RapierConfiguration {
        gravity: Vec3::NEG_Z * 9.81,
        ..default()
    });

    app.insert_resource(GameSettings {
        window_mode: WindowMode::Windowed,
        camera_sensitivity: 1.0,

        volume: 0.5,
        current_volume: 0.0,
        volume_change_timer: Timer::new(std::time::Duration::from_millis(50), TimerMode::Repeating),
    });

    app.run();
}

//                   |  Initial state
//                   |  GlobalState::AssetLoading
// Only resources    |  GameState::NotInGame
// are initialized   |  UiState::NoUi
//                   |
//                  ||->After asests are loader <-|
//                  ||  GlobalState::MainMenu     | Opinons are
//                  ||  GameState::NotInGame      | destroyed
// MainMenu         ||  UiState::MainMenu         |
// is destroyed     ||                            |
//                  ||->Pressing options ----------
//                  |   GlobalState::MainMenu
//                  |   GameState::NotInGame
//                  |   UiState::Options
// MainMenu         |
// is destroyed  ||||-> Pressing play           <-|
//               |||    GlobalState::InGame       |
//               |||    GameState::InGame         | Pause menu is
// HUD is        |||    UiState::Hud              | destroyed
// destroyed     |||                              |
//               |||->  Pressing pause in game ----           <-|
//               ||     GlobalState::InGame                     |
// Pause menu    ||     GameState::Paused                       | Options are
// is destroyed  ||     UiState::Paused                         | destroyed
//               ||                                             |
//               ||->   Pressing options while paused in game ---
//               |      GlobalState::InGame
// HUD is        |      GameState::Paused
// destroyed     |      UiState::Options
//               |
//               |->    Game over
//                      GlobalState::InGame
//                      GameState::GameOver
//                      UiState::GameOver

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, States)]
pub enum GlobalState {
    #[default]
    AssetLoading,
    MainMenu,
    InGame,
    Paused,
    GameOver,
    GameWon,
}
impl_into_state!(GlobalState);

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, States)]
pub enum UiState {
    #[default]
    NoUi,
    MainMenu,
    Options,
    Stats,
    Paused,
    GameOver,
    GameWon,
}
impl_into_state!(UiState);

#[derive(Resource)]
struct GameSettings {
    window_mode: WindowMode,
    camera_sensitivity: f32,

    volume: f32,
    current_volume: f32,
    volume_change_timer: Timer,
}
