use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::{
    enemies::{Enemy, EnemyAssets},
    player::{Player, PlayerResources},
    ui::UiResources,
    utils::remove_all_with,
    weapons::{Projectile, WeaponAssets},
    GlobalState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_PROJECTILES,
};

use self::{
    door::{Door, DoorAnimationFinished, DoorAnimationType},
    generation::{spawn_level, spawn_level_sun},
};

mod door;
mod generation;

const FLOOR_THICKNESS: f32 = 1.0;
const LEVEL_SIZE: f32 = 200.0;
const COLUMN_SIZE: f32 = 5.0;
const DOOR_THICKNESS: f32 = 2.0;
const COLUMN_HIGHT: f32 = 10.0;
const GRID_SIZE: usize = (LEVEL_SIZE / COLUMN_SIZE) as usize;
const FILL_AMOUNT: f32 = 0.02;
const STRIP_LENGTH: u32 = 3;

const LEVEL_WEAPON_SPAWNS: u32 = 4;
const LEVEL_ENEMIES: u32 = 4;

const LEVEL_LIGHTS_COVERAGE: f64 = 0.2;
const LIGHT_SIZE: f32 = 1.0;
const LIGHT_THICKENSS: f32 = 0.5;

const LEVEL_COLOR_NORMAL: Color = Color::WHITE;
const LEVEL_COLOR_ORANGE: Color = Color::ORANGE_RED;
const LEVEL_COLOR_BLUE: Color = Color::BLUE;
const LEVEL_COLOR_PINK: Color = Color::PINK;
const LEVEL_COLOR_GREEN: Color = Color::GREEN;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, LevelAssets>(GlobalState::AssetLoading);

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
                resume_physics,
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

#[derive(AssetCollection, Resource)]
pub struct LevelAssets {
    #[asset(path = "skyboxes/pink_skybox.png")]
    pub pink_skybox: Handle<Image>,
    #[asset(path = "skyboxes/orange_skybox.png")]
    pub orange_skybox: Handle<Image>,
    #[asset(path = "skyboxes/blue_skybox.png")]
    pub blue_skybox: Handle<Image>,
    #[asset(path = "skyboxes/normal_skybox.png")]
    pub normal_skybox: Handle<Image>,
    #[asset(path = "skyboxes/green_skybox.png")]
    pub green_skybox: Handle<Image>,
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
    light_mesh: Handle<Mesh>,
    light_material: Handle<StandardMaterial>,
}

// This component needs to be attached to
// all entities of the level. It will be
// used to clean up all entities from
// old level.
#[derive(Component)]
pub struct LevelObject;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LevelColor {
    Pink,
    Orange,
    Blue,
    Normal,
    Green,
}

impl LevelColor {
    fn skybox_image(&self, level_assets: &LevelAssets) -> Handle<Image> {
        match self {
            LevelColor::Pink => level_assets.pink_skybox.clone(),
            LevelColor::Orange => level_assets.orange_skybox.clone(),
            LevelColor::Blue => level_assets.blue_skybox.clone(),
            LevelColor::Normal => level_assets.normal_skybox.clone(),
            LevelColor::Green => level_assets.green_skybox.clone(),
        }
    }
}

impl From<LevelColor> for Color {
    fn from(value: LevelColor) -> Self {
        match value {
            LevelColor::Pink => LEVEL_COLOR_PINK,
            LevelColor::Orange => LEVEL_COLOR_ORANGE,
            LevelColor::Blue => LEVEL_COLOR_BLUE,
            LevelColor::Normal => LEVEL_COLOR_NORMAL,
            LevelColor::Green => LEVEL_COLOR_GREEN,
        }
    }
}

impl Distribution<LevelColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LevelColor {
        match rng.gen_range(0..5) {
            0 => LevelColor::Pink,
            1 => LevelColor::Orange,
            2 => LevelColor::Blue,
            3 => LevelColor::Normal,
            4 => LevelColor::Green,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LevelType {
    Covered,
    Open(LevelColor),
}

#[derive(Resource)]
struct LevelInfo {
    finished: bool,
    level_type: LevelType,
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

fn spawn_light(level_resources: &LevelResources, commands: &mut Commands, transform: Transform) {
    commands
        .spawn((
            PbrBundle {
                mesh: level_resources.light_mesh.clone(),
                material: level_resources.light_material.clone(),
                transform,
                ..default()
            },
            LevelObject,
        ))
        .with_children(|builder| {
            builder.spawn(PointLightBundle {
                point_light: PointLight {
                    intensity: 2000.0,
                    range: 100.0,
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.5)),
                ..default()
            });
        });
}

fn init_resources(
    level_assets: Res<LevelAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_mesh = meshes.add(shape::Box::new(LEVEL_SIZE, LEVEL_SIZE, FLOOR_THICKNESS).into());
    let floor_material = materials.add(Color::GRAY.into());

    let column_mesh = meshes.add(shape::Box::new(COLUMN_SIZE, COLUMN_SIZE, COLUMN_HIGHT).into());
    let column_material = materials.add(Color::DARK_GRAY.into());

    let door_mesh = meshes.add(shape::Box::new(COLUMN_SIZE, DOOR_THICKNESS, COLUMN_HIGHT).into());
    let door_closed_material = materials.add(Color::RED.into());
    let door_open_material = materials.add(Color::BLUE.into());

    let light_mesh = meshes.add(shape::Box::new(LIGHT_SIZE, LIGHT_SIZE, LIGHT_THICKENSS).into());
    let light_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE,
        ..default()
    });

    for handle in [
        &level_assets.pink_skybox,
        &level_assets.orange_skybox,
        &level_assets.blue_skybox,
        &level_assets.normal_skybox,
        &level_assets.green_skybox,
    ] {
        let skybox = images.get_mut(handle).unwrap();
        skybox.reinterpret_stacked_2d_as_array(skybox.height() / skybox.width());
        skybox.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    commands.insert_resource(LevelResources {
        floor_mesh,
        floor_material,
        column_mesh,
        column_material,
        door_mesh,
        door_closed_material,
        door_open_material,
        light_mesh,
        light_material,
    });
}

fn stop_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = false;
}

fn resume_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = true;
}

fn spawn_initial_level(
    ui_resources: Res<UiResources>,
    level_assets: Res<LevelAssets>,
    enemy_assets: Res<EnemyAssets>,
    weapon_assets: Res<WeaponAssets>,
    level_resources: Res<LevelResources>,
    player_resources: Res<PlayerResources>,
    mut commands: Commands,
) {
    spawn_level(
        ui_resources.as_ref(),
        level_assets.as_ref(),
        enemy_assets.as_ref(),
        weapon_assets.as_ref(),
        level_resources.as_ref(),
        player_resources.as_ref(),
        &mut commands,
        Vec3::ZERO,
        None,
        LevelType::Covered,
        true,
    );

    commands.insert_resource(LevelInfo {
        finished: false,
        level_type: LevelType::Covered,
        translation: Vec3::ZERO,
        old_level_objects: vec![],
    });
}

fn level_progress(
    enemies: Query<Entity, With<Enemy>>,
    mut level_state: ResMut<LevelInfo>,
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
    ui_resources: Res<UiResources>,
    level_assets: Res<LevelAssets>,
    enemy_assets: Res<EnemyAssets>,
    weapon_assets: Res<WeaponAssets>,
    level_resources: Res<LevelResources>,
    player_resources: Res<PlayerResources>,
    level_objects: Query<Entity, With<LevelObject>>,
    mut skybox: Query<&mut Skybox>,
    mut level_info: ResMut<LevelInfo>,
    mut commands: Commands,
    mut level_switch_events: EventReader<LevelSwitch>,
) {
    for event in level_switch_events.read() {
        let old_level_objects = level_objects.iter().collect::<Vec<_>>();

        let new_level_type = match level_info.level_type {
            LevelType::Open(_) => LevelType::Covered,
            LevelType::Covered => {
                if rand::random::<bool>() {
                    LevelType::Covered
                } else {
                    let level_color = rand::random::<LevelColor>();
                    LevelType::Open(level_color)
                }
            }
        };

        spawn_level_sun(new_level_type, &mut commands);
        if let Ok(mut skybox) = skybox.get_single_mut() {
            match new_level_type {
                LevelType::Covered => {}
                LevelType::Open(level_color) => {
                    skybox.0 = level_color.skybox_image(level_assets.as_ref());
                }
            }
        }

        let new_translation = spawn_level(
            ui_resources.as_ref(),
            level_assets.as_ref(),
            enemy_assets.as_ref(),
            weapon_assets.as_ref(),
            level_resources.as_ref(),
            player_resources.as_ref(),
            &mut commands,
            level_info.translation,
            Some(event.exit_door),
            new_level_type,
            false,
        );

        level_info.level_type = new_level_type;
        level_info.translation = new_translation;
        level_info.old_level_objects = old_level_objects;
    }
}

fn level_delete_old(
    mut commands: Commands,
    mut level_state: ResMut<LevelInfo>,
    mut door_amimation_finished_events: EventReader<DoorAnimationFinished>,
) {
    for animation_finished_event in door_amimation_finished_events.read() {
        match animation_finished_event.animation_type {
            DoorAnimationType::Open => {}
            DoorAnimationType::Close => {
                for object in level_state.old_level_objects.iter() {
                    if let Some(e) = commands.get_entity(*object) {
                        e.despawn_recursive();
                    }
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
