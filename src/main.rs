use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod level;
mod player;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        level::LevelPlugin,
        player::PlayerPlugin,
    ));

    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
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

    // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(50.0, 0.0, 10.0)
    //         .looking_at(Vec3::new(0.0, 0.0, 10.0), Vec3::Z),
    //     ..default()
    // });
}
