use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    damage::{Health, KillEvent},
    level::{LevelObject, LevelStarted},
    player::Player,
    weapons::{
        FreeFloatingWeaponBundle, ShootEvent, WeaponAssets, WeaponAttackTimer, WeaponBundle,
        WeaponModel,
    },
    GlobalState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PROJECTILES,
};

// Small enemy
const ENEMY_SMALL_COLLIDER_DIMENTION_X: f32 = 1.0;
const ENEMY_SMALL_COLLIDER_DIMENTION_Y: f32 = 1.0;
const ENEMY_SMALL_COLLIDER_DIMENTION_Z: f32 = 1.5;
const ENEMY_SMALL_DIMENTION_X: f32 = ENEMY_SMALL_COLLIDER_DIMENTION_X * 2.0;
const ENEMY_SMALL_DIMENTION_Y: f32 = ENEMY_SMALL_COLLIDER_DIMENTION_Y * 2.0;
const ENEMY_SMALL_DIMENTION_Z: f32 = ENEMY_SMALL_COLLIDER_DIMENTION_Z * 2.0;
const ENEMY_SMALL_PARTS_X: u32 = 2;
const ENEMY_SMALL_PARTS_Y: u32 = 2;
const ENEMY_SMALL_PARTS_Z: u32 = 2;
const ENEMY_SMALL_PART_DIMENTION_X: f32 = ENEMY_SMALL_DIMENTION_X / ENEMY_SMALL_PARTS_X as f32;
const ENEMY_SMALL_PART_DIMENTION_Y: f32 = ENEMY_SMALL_DIMENTION_Y / ENEMY_SMALL_PARTS_Y as f32;
const ENEMY_SMALL_PART_DIMENTION_Z: f32 = ENEMY_SMALL_DIMENTION_Z / ENEMY_SMALL_PARTS_Z as f32;

const ENEMY_SMALL_DEATH_GAP_X: f32 = 0.3;
const ENEMY_SMALL_DEATH_GAP_Y: f32 = 0.3;
const ENEMY_SMALL_DEATH_GAP_Z: f32 = 0.3;
const ENEMY_SMALL_DEATH_GAP_DELTA_X: f32 = ENEMY_SMALL_DEATH_GAP_X / ENEMY_SMALL_PARTS_X as f32;
const ENEMY_SMALL_DEATH_GAP_DELTA_Y: f32 = ENEMY_SMALL_DEATH_GAP_Y / ENEMY_SMALL_PARTS_Y as f32;
const ENEMY_SMALL_DEATH_GAP_DELTA_Z: f32 = ENEMY_SMALL_DEATH_GAP_Z / ENEMY_SMALL_PARTS_Z as f32;
const ENEMY_SMALL_DEATH_PULSE_STENGTH: f32 = 0.8;

const ENEMY_SMALL_HEALTH: i32 = 50;
const ENEMY_SMALL_SPEED: f32 = 20.0;
const ENEMY_SMALL_ROTATION_SPEED: f32 = 4.0;
const ENEMY_SMALL_MIN_DISTANCE: f32 = 400.0;
const ENEMY_SMALL_WEAPON_OFFSET: Vec3 = Vec3::new(1.0, 1.2, 0.5);

// Mid enemy
const ENEMY_MID_COLLIDER_DIMENTION_X: f32 = 1.0;
const ENEMY_MID_COLLIDER_DIMENTION_Y: f32 = 1.0;
const ENEMY_MID_COLLIDER_DIMENTION_Z: f32 = 2.5;
const ENEMY_MID_DIMENTION_X: f32 = ENEMY_MID_COLLIDER_DIMENTION_X * 2.0;
const ENEMY_MID_DIMENTION_Y: f32 = ENEMY_MID_COLLIDER_DIMENTION_Y * 2.0;
const ENEMY_MID_DIMENTION_Z: f32 = ENEMY_MID_COLLIDER_DIMENTION_Z * 2.0;
const ENEMY_MID_PARTS_X: u32 = 3;
const ENEMY_MID_PARTS_Y: u32 = 3;
const ENEMY_MID_PARTS_Z: u32 = 3;
const ENEMY_MID_PART_DIMENTION_X: f32 = ENEMY_MID_DIMENTION_X / ENEMY_MID_PARTS_X as f32;
const ENEMY_MID_PART_DIMENTION_Y: f32 = ENEMY_MID_DIMENTION_Y / ENEMY_MID_PARTS_Y as f32;
const ENEMY_MID_PART_DIMENTION_Z: f32 = ENEMY_MID_DIMENTION_Z / ENEMY_MID_PARTS_Z as f32;

const ENEMY_MID_DEATH_GAP_X: f32 = 0.1;
const ENEMY_MID_DEATH_GAP_Y: f32 = 0.1;
const ENEMY_MID_DEATH_GAP_Z: f32 = 0.1;
const ENEMY_MID_DEATH_GAP_DELTA_X: f32 = ENEMY_MID_DEATH_GAP_X / ENEMY_MID_PARTS_X as f32;
const ENEMY_MID_DEATH_GAP_DELTA_Y: f32 = ENEMY_MID_DEATH_GAP_Y / ENEMY_MID_PARTS_Y as f32;
const ENEMY_MID_DEATH_GAP_DELTA_Z: f32 = ENEMY_MID_DEATH_GAP_Z / ENEMY_MID_PARTS_Z as f32;
const ENEMY_MID_DEATH_PULSE_STENGTH: f32 = 0.8;

const ENEMY_MID_HEALTH: i32 = 100;
const ENEMY_MID_SPEED: f32 = 10.0;
const ENEMY_MID_ROTATION_SPEED: f32 = 2.0;
const ENEMY_MID_MIN_DISTANCE: f32 = 200.0;
const ENEMY_MID_WEAPON_OFFSET: Vec3 = Vec3::new(1.0, 1.2, 0.5);

// Big enemy
const ENEMY_BIG_COLLIDER_DIMENTION_X: f32 = 2.0;
const ENEMY_BIG_COLLIDER_DIMENTION_Y: f32 = 2.0;
const ENEMY_BIG_COLLIDER_DIMENTION_Z: f32 = 3.0;
const ENEMY_BIG_DIMENTION_X: f32 = ENEMY_BIG_COLLIDER_DIMENTION_X * 2.0;
const ENEMY_BIG_DIMENTION_Y: f32 = ENEMY_BIG_COLLIDER_DIMENTION_Y * 2.0;
const ENEMY_BIG_DIMENTION_Z: f32 = ENEMY_BIG_COLLIDER_DIMENTION_Z * 2.0;
const ENEMY_BIG_PARTS_X: u32 = 5;
const ENEMY_BIG_PARTS_Y: u32 = 5;
const ENEMY_BIG_PARTS_Z: u32 = 5;
const ENEMY_BIG_PART_DIMENTION_X: f32 = ENEMY_BIG_DIMENTION_X / ENEMY_BIG_PARTS_X as f32;
const ENEMY_BIG_PART_DIMENTION_Y: f32 = ENEMY_BIG_DIMENTION_Y / ENEMY_BIG_PARTS_Y as f32;
const ENEMY_BIG_PART_DIMENTION_Z: f32 = ENEMY_BIG_DIMENTION_Z / ENEMY_BIG_PARTS_Z as f32;

const ENEMY_BIG_DEATH_GAP_X: f32 = 0.3;
const ENEMY_BIG_DEATH_GAP_Y: f32 = 0.3;
const ENEMY_BIG_DEATH_GAP_Z: f32 = 0.3;
const ENEMY_BIG_DEATH_GAP_DELTA_X: f32 = ENEMY_BIG_DEATH_GAP_X / ENEMY_BIG_PARTS_X as f32;
const ENEMY_BIG_DEATH_GAP_DELTA_Y: f32 = ENEMY_BIG_DEATH_GAP_Y / ENEMY_BIG_PARTS_Y as f32;
const ENEMY_BIG_DEATH_GAP_DELTA_Z: f32 = ENEMY_BIG_DEATH_GAP_Z / ENEMY_BIG_PARTS_Z as f32;
const ENEMY_BIG_DEATH_PULSE_STENGTH: f32 = 1.8;

const ENEMY_BIG_HEALTH: i32 = 500;
const ENEMY_BIG_SPEED: f32 = 3.0;
const ENEMY_BIG_ROTATION_SPEED: f32 = 0.5;
const ENEMY_BIG_MIN_DISTANCE: f32 = 200.0;
const ENEMY_BIG_WEAPON_OFFSET: Vec3 = Vec3::new(2.0, 2.2, 0.5);

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, EnemyAssets>(GlobalState::AssetLoading);

        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            init_resources,
        );

        app.add_systems(
            Update,
            (enemy_enable, enemy_move, enemy_shoot, enemy_die)
                .run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct EnemyAssets {
    #[asset(path = "enemies/small_fridge.glb#Scene0")]
    pub small_enemy_scene: Handle<Scene>,
    #[asset(path = "enemies/mid_fridge.glb#Scene0")]
    pub mid_enemy_scene: Handle<Scene>,
    #[asset(path = "enemies/big_fridge.glb#Scene0")]
    pub big_enemy_scene: Handle<Scene>,
}

#[derive(Resource)]
pub struct EnemyResources {
    small_part_mesh: Handle<Mesh>,
    small_part_material: Handle<StandardMaterial>,
    mid_part_mesh: Handle<Mesh>,
    mid_part_material: Handle<StandardMaterial>,
    big_part_mesh: Handle<Mesh>,
    big_part_material: Handle<StandardMaterial>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Small,
    #[default]
    Mid,
    Big,
}

#[derive(Default, Component)]
pub struct Enemy {
    enemy_type: EnemyType,
    speed: f32,
    rotation_speed: f32,
    min_distance: f32,
    attached_weapon: Option<Entity>,
}

#[derive(Component)]
pub struct EnemyWeapon;

#[derive(Component)]
pub struct DisabledEnemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    rigid_body: RigidBody,
    collider: Collider,
    collision_groups: CollisionGroups,
    controller: KinematicCharacterController,
    locked_axis: LockedAxes,
    enemy: Enemy,

    scene_bundle: SceneBundle,
    health: Health,
    disabled: DisabledEnemy,

    level_object: LevelObject,
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
                filter_flags: QueryFilterFlags::EXCLUDE_SENSORS | QueryFilterFlags::EXCLUDE_DYNAMIC,
                ..default()
            },
            locked_axis: LockedAxes::TRANSLATION_LOCKED_Z,
            enemy: Enemy::default(),

            scene_bundle: SceneBundle::default(),
            health: Health::default(),
            disabled: DisabledEnemy,

            level_object: LevelObject,
        }
    }
}

fn init_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let small_part_mesh = meshes.add(
        shape::Box::new(
            ENEMY_SMALL_PART_DIMENTION_X,
            ENEMY_SMALL_PART_DIMENTION_Y,
            ENEMY_SMALL_PART_DIMENTION_Z,
        )
        .into(),
    );
    let small_part_material = materials.add(Color::YELLOW.into());

    let mid_part_mesh = meshes.add(
        shape::Box::new(
            ENEMY_MID_PART_DIMENTION_X,
            ENEMY_MID_PART_DIMENTION_Y,
            ENEMY_MID_PART_DIMENTION_Z,
        )
        .into(),
    );
    let mid_part_material = materials.add(Color::BLUE.into());

    let big_part_mesh = meshes.add(
        shape::Box::new(
            ENEMY_BIG_PART_DIMENTION_X,
            ENEMY_BIG_PART_DIMENTION_Y,
            ENEMY_BIG_PART_DIMENTION_Z,
        )
        .into(),
    );
    let big_part_material = materials.add(Color::RED.into());

    commands.insert_resource(EnemyResources {
        small_part_mesh,
        small_part_material,
        mid_part_mesh,
        mid_part_material,
        big_part_mesh,
        big_part_material,
    });
}

pub fn spawn_enemy(
    enemy_assets: &EnemyAssets,
    weapons_assets: &WeaponAssets,
    enemy_type: EnemyType,
    commands: &mut Commands,
    transform: Transform,
) {
    let (weapon_offset, health, collider, mut enemy, scene) = match enemy_type {
        EnemyType::Small => (
            ENEMY_SMALL_WEAPON_OFFSET,
            ENEMY_SMALL_HEALTH,
            Collider::cuboid(
                ENEMY_SMALL_COLLIDER_DIMENTION_X,
                ENEMY_SMALL_COLLIDER_DIMENTION_Y,
                ENEMY_SMALL_COLLIDER_DIMENTION_Z,
            ),
            Enemy {
                enemy_type,
                speed: ENEMY_SMALL_SPEED,
                rotation_speed: ENEMY_SMALL_ROTATION_SPEED,
                min_distance: ENEMY_SMALL_MIN_DISTANCE,
                attached_weapon: None,
            },
            enemy_assets.small_enemy_scene.clone(),
        ),
        EnemyType::Mid => (
            ENEMY_MID_WEAPON_OFFSET,
            ENEMY_MID_HEALTH,
            Collider::cuboid(
                ENEMY_MID_COLLIDER_DIMENTION_X,
                ENEMY_MID_COLLIDER_DIMENTION_Y,
                ENEMY_MID_COLLIDER_DIMENTION_Z,
            ),
            Enemy {
                enemy_type,
                speed: ENEMY_MID_SPEED,
                rotation_speed: ENEMY_MID_ROTATION_SPEED,
                min_distance: ENEMY_MID_MIN_DISTANCE,
                attached_weapon: None,
            },
            enemy_assets.mid_enemy_scene.clone(),
        ),
        EnemyType::Big => (
            ENEMY_BIG_WEAPON_OFFSET,
            ENEMY_BIG_HEALTH,
            Collider::cuboid(
                ENEMY_BIG_COLLIDER_DIMENTION_X,
                ENEMY_BIG_COLLIDER_DIMENTION_Y,
                ENEMY_BIG_COLLIDER_DIMENTION_Z,
            ),
            Enemy {
                enemy_type,
                speed: ENEMY_BIG_SPEED,
                rotation_speed: ENEMY_BIG_ROTATION_SPEED,
                min_distance: ENEMY_BIG_MIN_DISTANCE,
                attached_weapon: None,
            },
            enemy_assets.big_enemy_scene.clone(),
        ),
    };

    let weapon_transform = Transform::from_translation(weapon_offset);
    let weapon = match enemy_type {
        EnemyType::Small => commands
            .spawn((WeaponBundle::pistol(weapon_transform), EnemyWeapon))
            .with_children(|builder| {
                builder.spawn((
                    SceneBundle {
                        scene: weapons_assets.pistol_scene.clone(),
                        ..default()
                    },
                    WeaponModel,
                ));
            })
            .id(),
        EnemyType::Mid => commands
            .spawn((WeaponBundle::shotgun(weapon_transform), EnemyWeapon))
            .with_children(|builder| {
                builder.spawn((
                    SceneBundle {
                        scene: weapons_assets.shotgun_scene.clone(),
                        ..default()
                    },
                    WeaponModel,
                ));
            })
            .id(),
        EnemyType::Big => commands
            .spawn((WeaponBundle::minigun(weapon_transform), EnemyWeapon))
            .with_children(|builder| {
                builder.spawn((
                    SceneBundle {
                        scene: weapons_assets.minigun_scene.clone(),
                        ..default()
                    },
                    WeaponModel,
                ));
            })
            .id(),
    };

    enemy.attached_weapon = Some(weapon);
    commands
        .spawn(EnemyBundle {
            scene_bundle: SceneBundle {
                scene,
                transform: transform.with_scale(Vec3::new(1.5, 1.5, 1.5)),
                ..default()
            },
            enemy,
            health: Health { health },
            collider,
            ..default()
        })
        .add_child(weapon);
}

fn enemy_enable(
    enemies: Query<Entity, With<DisabledEnemy>>,
    mut commands: Commands,
    mut level_started_events: EventReader<LevelStarted>,
) {
    for _ in level_started_events.read() {
        for enemy in enemies.iter() {
            commands
                .get_entity(enemy)
                .unwrap()
                .remove::<DisabledEnemy>();
        }
    }
}

#[allow(clippy::complexity)]
fn enemy_move(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<
        (&Enemy, &mut Transform, &mut KinematicCharacterController),
        (Without<DisabledEnemy>, Without<Player>),
    >,
) {
    let Ok(player_transfomr) = player.get_single() else {
        return;
    };

    for (enemy, mut enemy_transform, mut enemy_controller) in enemies.iter_mut() {
        let v = player_transfomr.translation.xy() - enemy_transform.translation.xy();
        let direction = v.normalize();
        if enemy.min_distance < v.length_squared() {
            let movement = direction * enemy.speed * time.delta_seconds();
            enemy_controller.translation = Some(movement.extend(0.0));
        }

        let direction = direction.extend(0.0);
        let enemy_forward = enemy_transform.rotation * Vec3::Y;
        let mut angle = direction.angle_between(enemy_forward);
        let cross = direction.cross(enemy_forward);
        if 0.0 <= cross.z {
            angle *= -1.0;
        }
        let target_rotation = enemy_transform.rotation * Quat::from_rotation_z(angle);
        enemy_transform.rotation = enemy_transform
            .rotation
            .lerp(target_rotation, enemy.rotation_speed * time.delta_seconds());
    }
}

fn enemy_shoot(
    rapier_context: Res<RapierContext>,
    player: Query<Entity, With<Player>>,
    mut enemy_weapons: Query<(Entity, &GlobalTransform, &mut WeaponAttackTimer), With<EnemyWeapon>>,
    mut shoot_event: EventWriter<ShootEvent>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    for (weapon_entity, weapon_global_transform, mut weapon_attack_timer) in
        enemy_weapons.iter_mut()
    {
        let ray_dir = weapon_global_transform.up();
        let ray_origin = weapon_global_transform.translation();
        let max_toi = 300.0;
        let solid = true;
        let filter = QueryFilter {
            flags: QueryFilterFlags::EXCLUDE_SENSORS,
            ..default()
        };
        if let Some((entity, _)) =
            rapier_context.cast_ray(ray_origin, ray_dir, max_toi, solid, filter)
        {
            if entity == player && weapon_attack_timer.ready {
                weapon_attack_timer.attack_timer.reset();
                weapon_attack_timer.ready = false;
                shoot_event.send(ShootEvent {
                    weapon_entity,
                    weapon_translation: weapon_global_transform.translation(),
                    direction: weapon_global_transform.up(),
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_parts(
    parts_x: u32,
    parts_y: u32,
    parts_z: u32,
    dimention_x: f32,
    dimention_y: f32,
    dimention_z: f32,
    part_dimention_x: f32,
    part_dimention_y: f32,
    part_dimention_z: f32,
    gap_x: f32,
    gap_y: f32,
    gap_z: f32,
    gap_delta_x: f32,
    gap_delta_y: f32,
    gap_delta_z: f32,
    pulse_strength: f32,
    part_mesh: Handle<Mesh>,
    part_material: Handle<StandardMaterial>,
    enemy_transform: Transform,
    commands: &mut Commands,
) {
    for x in 0..parts_x {
        for y in 0..parts_y {
            for z in 0..parts_z {
                let x_pos =
                    -(dimention_x + gap_x) / 2.0 + (part_dimention_x + gap_delta_x) * x as f32;
                let y_pos =
                    -(dimention_y + gap_y) / 2.0 + (part_dimention_y + gap_delta_y) * y as f32;
                let z_pos = -(dimention_z + gap_z) / 2.0
                            + (part_dimention_z + gap_delta_z) * z as f32
                            // to make all parts be above ground
                            + dimention_z / 2.0;
                let pos = Vec3::new(x_pos, y_pos, z_pos);
                let translation = enemy_transform.transform_point(pos);
                let transform = Transform::from_translation(translation)
                    .with_rotation(enemy_transform.rotation);
                let linvel =
                    (translation - enemy_transform.translation).normalize() * pulse_strength;
                commands.spawn((
                    PbrBundle {
                        mesh: part_mesh.clone(),
                        material: part_material.clone(),
                        transform,
                        ..default()
                    },
                    Collider::cuboid(
                        part_dimention_x / 2.0,
                        part_dimention_y / 2.0,
                        part_dimention_z / 2.0,
                    ),
                    RigidBody::Dynamic,
                    Velocity {
                        linvel,
                        ..default()
                    },
                    LevelObject,
                ));
            }
        }
    }
}

fn enemy_die(
    enemy_resources: Res<EnemyResources>,
    enemies: Query<(Entity, &Transform, &Enemy), Without<EnemyWeapon>>,
    mut weapons: Query<(Entity, &mut Transform), With<EnemyWeapon>>,
    mut commands: Commands,
    mut kill_events: EventReader<KillEvent>,
) {
    for kill_event in kill_events.read() {
        if let Ok((enemy_entity, enemy_transform, enemy)) = enemies.get(kill_event.entity) {
            match enemy.enemy_type {
                EnemyType::Small => spawn_parts(
                    ENEMY_SMALL_PARTS_X,
                    ENEMY_SMALL_PARTS_Y,
                    ENEMY_SMALL_PARTS_Z,
                    ENEMY_SMALL_DIMENTION_X,
                    ENEMY_SMALL_DIMENTION_Y,
                    ENEMY_SMALL_DIMENTION_Z,
                    ENEMY_SMALL_PART_DIMENTION_X,
                    ENEMY_SMALL_PART_DIMENTION_Y,
                    ENEMY_SMALL_PART_DIMENTION_Z,
                    ENEMY_SMALL_DEATH_GAP_X,
                    ENEMY_SMALL_DEATH_GAP_Y,
                    ENEMY_SMALL_DEATH_GAP_Z,
                    ENEMY_SMALL_DEATH_GAP_DELTA_X,
                    ENEMY_SMALL_DEATH_GAP_DELTA_Y,
                    ENEMY_SMALL_DEATH_GAP_DELTA_Z,
                    ENEMY_SMALL_DEATH_PULSE_STENGTH,
                    enemy_resources.small_part_mesh.clone(),
                    enemy_resources.small_part_material.clone(),
                    *enemy_transform,
                    &mut commands,
                ),
                EnemyType::Mid => spawn_parts(
                    ENEMY_MID_PARTS_X,
                    ENEMY_MID_PARTS_Y,
                    ENEMY_MID_PARTS_Z,
                    ENEMY_MID_DIMENTION_X,
                    ENEMY_MID_DIMENTION_Y,
                    ENEMY_MID_DIMENTION_Z,
                    ENEMY_MID_PART_DIMENTION_X,
                    ENEMY_MID_PART_DIMENTION_Y,
                    ENEMY_MID_PART_DIMENTION_Z,
                    ENEMY_MID_DEATH_GAP_X,
                    ENEMY_MID_DEATH_GAP_Y,
                    ENEMY_MID_DEATH_GAP_Z,
                    ENEMY_MID_DEATH_GAP_DELTA_X,
                    ENEMY_MID_DEATH_GAP_DELTA_Y,
                    ENEMY_MID_DEATH_GAP_DELTA_Z,
                    ENEMY_MID_DEATH_PULSE_STENGTH,
                    enemy_resources.mid_part_mesh.clone(),
                    enemy_resources.mid_part_material.clone(),
                    *enemy_transform,
                    &mut commands,
                ),
                EnemyType::Big => spawn_parts(
                    ENEMY_BIG_PARTS_X,
                    ENEMY_BIG_PARTS_Y,
                    ENEMY_BIG_PARTS_Z,
                    ENEMY_BIG_DIMENTION_X,
                    ENEMY_BIG_DIMENTION_Y,
                    ENEMY_BIG_DIMENTION_Z,
                    ENEMY_BIG_PART_DIMENTION_X,
                    ENEMY_BIG_PART_DIMENTION_Y,
                    ENEMY_BIG_PART_DIMENTION_Z,
                    ENEMY_BIG_DEATH_GAP_X,
                    ENEMY_BIG_DEATH_GAP_Y,
                    ENEMY_BIG_DEATH_GAP_Z,
                    ENEMY_BIG_DEATH_GAP_DELTA_X,
                    ENEMY_BIG_DEATH_GAP_DELTA_Y,
                    ENEMY_BIG_DEATH_GAP_DELTA_Z,
                    ENEMY_BIG_DEATH_PULSE_STENGTH,
                    enemy_resources.big_part_mesh.clone(),
                    enemy_resources.big_part_material.clone(),
                    *enemy_transform,
                    &mut commands,
                ),
            }

            // drop weapon
            if let Some(attached_weapon) = enemy.attached_weapon {
                if let Ok((weapon, mut weapon_transform)) = weapons.get_mut(attached_weapon) {
                    commands
                        .get_entity(enemy_entity)
                        .unwrap()
                        .remove_children(&[weapon]);
                    *weapon_transform = *enemy_transform;
                    commands
                        .get_entity(weapon)
                        .unwrap()
                        .remove::<EnemyWeapon>()
                        .insert(FreeFloatingWeaponBundle::new(enemy_transform.translation));
                }
            }

            commands
                .get_entity(enemy_entity)
                .unwrap()
                .despawn_recursive();
        }
    }
}
