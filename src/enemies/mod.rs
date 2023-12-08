use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    GlobalState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES,
};

use self::fridge::{FRIDGE_PART_DIMENTION_X, FRIDGE_PART_DIMENTION_Y, FRIDGE_PART_DIMENTION_Z};

pub mod fridge;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, EnemyAssets>(GlobalState::AssetLoading);

        app.add_plugins(fridge::FridgePlugin);

        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            init_resources,
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct EnemyAssets {
    #[asset(path = "enemies/small_fridge.glb#Scene0")]
    pub small_fridge_scene: Handle<Scene>,
    #[asset(path = "enemies/mid_fridge.glb#Scene0")]
    pub mid_fridge_scene: Handle<Scene>,
    #[asset(path = "enemies/big_fridge.glb#Scene0")]
    pub big_fridge_scene: Handle<Scene>,
}

#[derive(Resource)]
pub struct EnemyResources {
    fridge_part_mesh: Handle<Mesh>,
    fridge_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    rigid_body: RigidBody,
    collider: Collider,
    collision_groups: CollisionGroups,
    controller: KinematicCharacterController,
    locked_axis: LockedAxes,
    enemy: Enemy,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_ENEMY,
                COLLISION_GROUP_LEVEL | COLLISION_GROUP_PROJECTILES,
            ),
            controller: KinematicCharacterController {
                up: Vec3::Z,
                offset: CharacterLength::Relative(0.1),
                ..default()
            },
            locked_axis: LockedAxes::TRANSLATION_LOCKED_Z,
            enemy: Enemy,
        }
    }
}

fn init_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let fridge_part_mesh = meshes.add(
        shape::Box::new(
            FRIDGE_PART_DIMENTION_X,
            FRIDGE_PART_DIMENTION_Y,
            FRIDGE_PART_DIMENTION_Z,
        )
        .into(),
    );
    let fridge_material = materials.add(Color::WHITE.into());

    commands.insert_resource(EnemyResources {
        fridge_part_mesh,
        fridge_material,
    });
}
