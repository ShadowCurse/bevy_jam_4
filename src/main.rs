use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // floor
    let mesh = meshes.add(shape::Box::new(100.0, 100.0, 1.0).into());
    let material = materials.add(Color::GRAY.into());
    commands.spawn(PbrBundle {
        mesh,
        material,
        ..default()
    });
}
