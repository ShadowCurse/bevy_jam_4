use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;

mod damage;
mod enemies;
mod level;
mod player;
mod weapons;

const COLLISION_GROUP_LEVEL: Group = Group::GROUP_1;
const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
const COLLISION_GROUP_ENEMY: Group = Group::GROUP_3;
const COLLISION_GROUP_PROJECTILES: Group = Group::GROUP_4;
const COLLISION_GROUP_PICKUP: Group = Group::GROUP_5;

fn main() {
    let mut app = App::new();

    app.add_state::<GlobalState>();
    app.add_state::<GameState>();
    app.add_state::<UiState>();

    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        damage::DamagePlugin,
        enemies::EnemiesPlugin,
        level::LevelPlugin,
        player::PlayerPlugin,
        weapons::WeaponsPlugin,
    ));

    app.add_loading_state(
        LoadingState::new(GlobalState::AssetLoading).continue_to_state(GlobalState::MainMenu),
    );

    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    app.insert_resource(RapierConfiguration {
        gravity: Vec3::NEG_Z * 9.81,
        ..default()
    });

    app.add_systems(
        OnTransition {
            from: GlobalState::AssetLoading,
            to: GlobalState::MainMenu,
        },
        setup,
    );
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
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    NotInGame,
    InGame,
    Paused,
    GameOver,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, States)]
pub enum UiState {
    #[default]
    NoUi,
    MainMenu,
    Options,
    Hud,
    Paused,
    GameOver,
}

fn setup(
    // mut commands: Commands,
    mut global_state: ResMut<NextState<GlobalState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    global_state.set(GlobalState::InGame);
    game_state.set(GameState::InGame);
}
