use bevy::prelude::*;
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

    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    app.insert_resource(RapierConfiguration {
        gravity: Vec3::NEG_Z * 9.81,
        ..default()
    });

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 30.0),
        ..default()
    });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
        ..default()
    });

    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(0.0, 0.0, 300.0)
    //         .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
    //     ..default()
    // });
}
