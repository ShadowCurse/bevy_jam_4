use bevy::{prelude::*, sprite::collide_aabb::Collision};
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};

use crate::{
    weapons::Projectile, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
        app.add_systems(PostStartup, generate_level);
        app.add_systems(Update, collision_level_object_projectiles);
    }
}

#[derive(Component)]
pub struct LevelObject;

#[derive(Resource)]
struct LevelResources {
    floor_material: Handle<StandardMaterial>,
    wall_material: Handle<StandardMaterial>,
}

fn init(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let floor_material = materials.add(Color::GRAY.into());
    let wall_material = materials.add(Color::DARK_GRAY.into());

    commands.insert_resource(LevelResources {
        floor_material,
        wall_material,
    });
}

fn generate_level(
    level_resources: Res<LevelResources>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // floor
    let dimentions = Vec3::new(500.0, 500.0, 1.0);
    let mesh = meshes.add(shape::Box::new(dimentions.x, dimentions.y, dimentions.z).into());
    commands.spawn((
        PbrBundle {
            mesh,
            material: level_resources.floor_material.clone(),
            ..default()
        },
        Collider::cuboid(dimentions.x / 2.0, dimentions.y / 2.0, dimentions.z / 2.0),
        RigidBody::Fixed,
        LevelObject,
    ));

    // +X test wall
    let dimentions = Vec3::new(1.0, 100.0, 10.0);
    let mesh = meshes.add(shape::Box::new(dimentions.x, dimentions.y, dimentions.z).into());
    let transform = Transform::from_translation(Vec3::new(20.0, 0.0, 5.0));
    commands.spawn((
        PbrBundle {
            mesh: mesh.clone(),
            material: level_resources.wall_material.clone(),
            transform,
            ..default()
        },
        Collider::cuboid(dimentions.x / 2.0, dimentions.y / 2.0, dimentions.z / 2.0),
        CollisionGroups::new(
            COLLISION_GROUP_LEVEL,
            COLLISION_GROUP_ENEMY | COLLISION_GROUP_PROJECTILES,
        ),
        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
        RigidBody::Fixed,
        LevelObject,
    ));

    // +X wall
    let dimentions = Vec3::new(1.0, 500.0, 10.0);
    let mesh = meshes.add(shape::Box::new(dimentions.x, dimentions.y, dimentions.z).into());
    let transform = Transform::from_translation(Vec3::new(250.0, 0.0, 5.0));
    commands.spawn((
        PbrBundle {
            mesh: mesh.clone(),
            material: level_resources.wall_material.clone(),
            transform,
            ..default()
        },
        Collider::cuboid(dimentions.x / 2.0, dimentions.y / 2.0, dimentions.z / 2.0),
        CollisionGroups::new(COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES),
        RigidBody::Fixed,
        LevelObject,
    ));

    // -X wall
    let transform = Transform::from_translation(Vec3::new(-250.0, 0.0, 5.0));
    commands.spawn((
        PbrBundle {
            mesh,
            material: level_resources.wall_material.clone(),
            transform,
            ..default()
        },
        Collider::cuboid(dimentions.x / 2.0, dimentions.y / 2.0, dimentions.z / 2.0),
        CollisionGroups::new(COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES),
        RigidBody::Fixed,
        LevelObject,
    ));

    // +Y wall
    let dimentions = Vec3::new(500.0, 1.0, 10.0);
    let mesh = meshes.add(shape::Box::new(dimentions.x, dimentions.y, dimentions.z).into());
    let transform = Transform::from_translation(Vec3::new(0.0, 250.0, 5.0));
    commands.spawn((
        PbrBundle {
            mesh: mesh.clone(),
            material: level_resources.wall_material.clone(),
            transform,
            ..default()
        },
        Collider::cuboid(dimentions.x / 2.0, dimentions.y / 2.0, dimentions.z / 2.0),
        CollisionGroups::new(COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES),
        RigidBody::Fixed,
        LevelObject,
    ));

    // -Y wall
    let transform = Transform::from_translation(Vec3::new(0.0, -250.0, 5.0));
    commands.spawn((
        PbrBundle {
            mesh,
            material: level_resources.wall_material.clone(),
            transform,
            ..default()
        },
        Collider::cuboid(dimentions.x / 2.0, dimentions.y / 2.0, dimentions.z / 2.0),
        CollisionGroups::new(COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES),
        RigidBody::Fixed,
        LevelObject,
    ));
}

fn collision_level_object_projectiles(
    level_objects: Query<Entity, With<LevelObject>>,
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        let (collider_1, collider_2, flags) = match collision_event {
            CollisionEvent::Started(c1, c2, f) => (c1, c2, f),
            CollisionEvent::Stopped(c1, c2, f) => (c1, c2, f),
        };
        if flags.contains(CollisionEventFlags::REMOVED) {
            return;
        }
        let (contains_1, contains_2) = (
            level_objects.contains(*collider_1),
            level_objects.contains(*collider_2),
        );
        if contains_1 {
            commands.get_entity(*collider_2).unwrap().despawn();
        } else if contains_2 {
            commands.get_entity(*collider_1).unwrap().despawn();
        }
    }
}
