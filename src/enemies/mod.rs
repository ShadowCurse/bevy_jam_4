use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use self::fridge::{
    FRIDGE_DIMENTION_X, FRIDGE_DIMENTION_Y, FRIDGE_DIMENTION_Z, FRIDGE_PART_DIMENTION_X,
    FRIDGE_PART_DIMENTION_Y, FRIDGE_PART_DIMENTION_Z,
};

pub mod fridge;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_resources);
        app.add_plugins(fridge::FridgePlugin);
    }
}

#[derive(Resource)]
pub struct EnemiesResources {
    fridge_mesh: Handle<Mesh>,
    fridge_part_mesh: Handle<Mesh>,
    fridge_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pbr: PbrBundle,
    collider: Collider,
    rigid_body: RigidBody,
    velocity: Velocity,
    enemy: Enemy,
}

impl EnemyBundle {
    pub fn new(transform: Transform, enemies_resources: &EnemiesResources) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: enemies_resources.fridge_mesh.clone(),
                material: enemies_resources.fridge_material.clone(),
                transform,
                ..default()
            },
            collider: Collider::capsule(Vec3::new(0.0, 0.0, -3.5), Vec3::new(0.0, 0.0, 3.5), 2.0),
            rigid_body: RigidBody::KinematicVelocityBased,
            velocity: Velocity::default(),
            enemy: Enemy,
        }
    }
}

fn init_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // forward = -Z
    let fridge_mesh = meshes
        .add(shape::Box::new(FRIDGE_DIMENTION_X, FRIDGE_DIMENTION_Y, FRIDGE_DIMENTION_Z).into());
    let fridge_part_mesh = meshes.add(
        shape::Box::new(
            FRIDGE_PART_DIMENTION_X,
            FRIDGE_PART_DIMENTION_Y,
            FRIDGE_PART_DIMENTION_Z,
        )
        .into(),
    );
    let fridge_material = materials.add(Color::WHITE.into());

    commands.insert_resource(EnemiesResources {
        fridge_mesh,
        fridge_part_mesh,
        fridge_material,
    });
}
