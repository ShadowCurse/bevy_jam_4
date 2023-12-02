use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod level;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        level::LevelPlugin,
    ));

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 30.0),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 20.0)
            .looking_at(Vec3::new(50.0, 0.0, 0.0), Vec3::Z),
        ..default()
    });
}
