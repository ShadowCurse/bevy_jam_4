use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};

use crate::{
    enemies::{EnemiesResources, Enemy},
    player::Player,
    utils::remove_all_with,
    weapons::{Projectile, WeaponsResources},
    GlobalState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_PROJECTILES,
};

use self::{
    door::{Door, DoorAnimationFinished, DoorAnimationType},
    generation::{spawn_level, spawn_level_sun},
};

mod door;
mod generation;

const LEVEL_SIZE: f32 = 200.0;
const COLUMN_SIZE: f32 = 5.0;
const DOOR_THICKNESS: f32 = 2.0;
const COLUMN_HIGHT: f32 = 10.0;
const GRID_SIZE: usize = (LEVEL_SIZE / COLUMN_SIZE) as usize;
const FILL_AMOUNT: f32 = 0.02;
const STRIP_LENGTH: u32 = 3;

const LEVEL_WEAPON_SPAWNS: u32 = 4;
const LEVEL_ENEMIES: u32 = 1;

const LIGHT_COLORS: [Color; 3] = [Color::WHITE, Color::BLUE, Color::ORANGE_RED];

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelStarted>();
        app.add_event::<LevelFinished>();
        app.add_event::<LevelSwitch>();

        app.add_plugins(door::DoorPlugin);

        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            init_resources,
        );

        app.add_systems(
            OnEnter(GlobalState::MainMenu),
            (resume_physics, spawn_initial_level),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::Paused,
            },
            stop_physics,
        );
        app.add_systems(
            OnTransition {
                from: GlobalState::InGame,
                to: GlobalState::GameOver,
            },
            stop_physics,
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::Paused,
                to: GlobalState::InGame,
            },
            resume_physics,
        );
        app.add_systems(
            OnTransition {
                from: GlobalState::Paused,
                to: GlobalState::MainMenu,
            },
            (remove_all_with::<LevelObject>, remove_all_with::<Player>),
        );

        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::MainMenu,
            },
            (remove_all_with::<LevelObject>, remove_all_with::<Player>),
        );
        app.add_systems(
            OnTransition {
                from: GlobalState::GameOver,
                to: GlobalState::InGame,
            },
            (
                remove_all_with::<LevelObject>,
                remove_all_with::<Player>,
                spawn_initial_level,
            )
                .chain(),
        );

        app.add_systems(
            Update,
            (
                level_progress,
                level_switch,
                level_delete_old,
                collision_level_object_projectiles,
            )
                .run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(Resource)]
struct LevelResources {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,
    column_mesh: Handle<Mesh>,
    column_material: Handle<StandardMaterial>,
    door_mesh: Handle<Mesh>,
    door_closed_material: Handle<StandardMaterial>,
    door_open_material: Handle<StandardMaterial>,
}

// This component needs to be attached to
// all entities of the level. It will be
// used to clean up all entities from
// old level.
#[derive(Component)]
pub struct LevelObject;

#[derive(Resource)]
struct LevelState {
    finished: bool,
    translation: Vec3,
    old_level_objects: Vec<Entity>,
}

#[derive(Event)]
pub struct LevelStarted;

#[derive(Event)]
pub struct LevelFinished;

#[derive(Event)]
pub struct LevelSwitch {
    exit_door: Door,
}

#[derive(Component)]
pub struct LevelCollider;

#[derive(Bundle)]
pub struct LevelColliderBundle {
    pub pbr_bundle: PbrBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_collision_types: ActiveCollisionTypes,
    pub rigid_body: RigidBody,
    pub level_collider: LevelCollider,

    pub level_object: LevelObject,
}

impl Default for LevelColliderBundle {
    fn default() -> Self {
        Self {
            pbr_bundle: PbrBundle::default(),
            collider: Collider::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_LEVEL,
                COLLISION_GROUP_ENEMY | COLLISION_GROUP_PLAYER | COLLISION_GROUP_PROJECTILES,
            ),
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            rigid_body: RigidBody::Fixed,
            level_collider: LevelCollider,

            level_object: LevelObject,
        }
    }
}

impl LevelColliderBundle {
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        collider: Collider,
    ) -> Self {
        Self {
            pbr_bundle: PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            collider,
            ..default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellType {
    Empty,
    Door(Door),
    Column,
    Weapon,
    Enemy,
    Player,
}

fn init_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_mesh = meshes.add(shape::Box::new(LEVEL_SIZE, LEVEL_SIZE, 1.0).into());
    let floor_material = materials.add(Color::GRAY.into());

    let column_mesh = meshes.add(shape::Box::new(COLUMN_SIZE, COLUMN_SIZE, COLUMN_HIGHT).into());
    let column_material = materials.add(Color::DARK_GRAY.into());

    let door_mesh = meshes.add(shape::Box::new(COLUMN_SIZE, DOOR_THICKNESS, COLUMN_HIGHT).into());
    let door_closed_material = materials.add(Color::RED.into());
    let door_open_material = materials.add(Color::BLUE.into());

    commands.insert_resource(LevelResources {
        floor_mesh,
        floor_material,
        column_mesh,
        column_material,
        door_mesh,
        door_closed_material,
        door_open_material,
    });
}

fn stop_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = false;
}

fn resume_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = true;
}

fn spawn_initial_level(
    level_resources: Res<LevelResources>,
    weapons_resources: Res<WeaponsResources>,
    enemies_resources: Res<EnemiesResources>,
    mut commands: Commands,
) {
    spawn_level(
        level_resources.as_ref(),
        weapons_resources.as_ref(),
        enemies_resources.as_ref(),
        &mut commands,
        Vec3::ZERO,
        None,
        true,
    );
    spawn_level_sun(&mut commands);

    commands.insert_resource(LevelState {
        finished: false,
        translation: Vec3::ZERO,
        old_level_objects: vec![],
    });
}

fn level_progress(
    enemies: Query<Entity, With<Enemy>>,
    mut level_state: ResMut<LevelState>,
    mut level_started_events: EventReader<LevelStarted>,
    mut level_finished_events: EventWriter<LevelFinished>,
) {
    for _ in level_started_events.read() {
        level_state.finished = false;
    }

    let remaining_enemies = enemies.iter().count();
    if remaining_enemies == 0 && !level_state.finished {
        level_state.finished = true;
        level_finished_events.send(LevelFinished);
    }
}

fn level_switch(
    level_resources: Res<LevelResources>,
    weapons_resources: Res<WeaponsResources>,
    enemies_resources: Res<EnemiesResources>,
    level_objects: Query<Entity, With<LevelObject>>,
    mut level_state: ResMut<LevelState>,
    mut commands: Commands,
    mut level_switch_events: EventReader<LevelSwitch>,
) {
    for event in level_switch_events.read() {
        let old_level_objects = level_objects.iter().collect::<Vec<_>>();

        let new_translation = spawn_level(
            level_resources.as_ref(),
            weapons_resources.as_ref(),
            enemies_resources.as_ref(),
            &mut commands,
            level_state.translation,
            Some(event.exit_door),
            false,
        );
        spawn_level_sun(&mut commands);

        level_state.translation = new_translation;
        level_state.old_level_objects = old_level_objects;
    }
}

fn level_delete_old(
    mut commands: Commands,
    mut level_state: ResMut<LevelState>,
    mut door_amimation_finished_events: EventReader<DoorAnimationFinished>,
) {
    for animation_finished_event in door_amimation_finished_events.read() {
        match animation_finished_event.animation_type {
            DoorAnimationType::Open => {}
            DoorAnimationType::Close => {
                for object in level_state.old_level_objects.iter() {
                    commands.get_entity(*object).unwrap().despawn_recursive();
                }
                level_state.old_level_objects.clear();
            }
        }
    }
}

fn collision_level_object_projectiles(
    projectiles: Query<Entity, With<Projectile>>,
    level_objects: Query<Entity, With<LevelCollider>>,
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

        let projectile = if let Ok(p) = projectiles.get(*collider_1) {
            if level_objects.get(*collider_2).is_ok() {
                p
            } else {
                continue;
            }
        } else if let Ok(p) = projectiles.get(*collider_2) {
            if level_objects.get(*collider_1).is_ok() {
                p
            } else {
                continue;
            }
        } else {
            continue;
        };
        commands.get_entity(projectile).unwrap().despawn();
    }
}
