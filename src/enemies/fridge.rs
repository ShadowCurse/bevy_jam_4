use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    damage::{Health, KillEvent},
    level::{LevelObject, LevelStarted},
    player::Player,
    weapons::{
        pistol::PistolBundle, FreeFloatingWeaponBundle, ShootEvent, WeaponAttackTimer,
        WeaponsResources,
    },
    GameState,
};

use super::{EnemiesResources, EnemyBundle};

pub const FRIDGE_DIMENTION_X: f32 = 3.5;
pub const FRIDGE_DIMENTION_Y: f32 = 3.5;
pub const FRIDGE_DIMENTION_Z: f32 = 7.0;
pub const FRIDGE_PARTS_X: u32 = 2;
pub const FRIDGE_PARTS_Y: u32 = 2;
pub const FRIDGE_PARTS_Z: u32 = 2;
pub const FRIDGE_PART_DIMENTION_X: f32 = FRIDGE_DIMENTION_X / FRIDGE_PARTS_X as f32;
pub const FRIDGE_PART_DIMENTION_Y: f32 = FRIDGE_DIMENTION_Y / FRIDGE_PARTS_Y as f32;
pub const FRIDGE_PART_DIMENTION_Z: f32 = FRIDGE_DIMENTION_Z / FRIDGE_PARTS_Z as f32;
const FRIDGE_DEATH_GAP_X: f32 = 0.3;
const FRIDGE_DEATH_GAP_Y: f32 = 0.3;
const FRIDGE_DEATH_GAP_Z: f32 = 0.3;
const FRIDGE_DEATH_GAP_DELTA_X: f32 = FRIDGE_DEATH_GAP_X / FRIDGE_PARTS_X as f32;
const FRIDGE_DEATH_GAP_DELTA_Y: f32 = FRIDGE_DEATH_GAP_Y / FRIDGE_PARTS_Y as f32;
const FRIDGE_DEATH_GAP_DELTA_Z: f32 = FRIDGE_DEATH_GAP_Z / FRIDGE_PARTS_Z as f32;
const FRIDGE_DEATH_PULSE_STENGTH: f32 = 0.8;

const FRIDGE_HEALTH: i32 = 100;
const FRIDGE_SPEED: f32 = 15.0;
const FRIDGE_MIN_DISTANCE: f32 = 200.0;
const FRIDGE_WEAPON_OFFSET: Vec3 = Vec3::new(2.0, -2.0, 0.5);

pub struct FridgePlugin;

impl Plugin for FridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (fridge_enable, fridge_move, fridge_shoot, fridge_die)
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Fridge {
    attached_weapon: Option<Entity>,
}

#[derive(Component)]
pub struct FridgeDisabled;

#[derive(Component)]
pub struct FridgeWeapon;

#[derive(Bundle)]
pub struct FridgeBuldle {
    enemy_bundle: EnemyBundle,
    health: Health,
    fridge: Fridge,
    disabled: FridgeDisabled,

    level_object: LevelObject,
}

impl FridgeBuldle {
    pub fn new(
        health: i32,
        attached_weapon: Option<Entity>,
        transform: Transform,
        enemies_resources: &EnemiesResources,
    ) -> Self {
        Self {
            enemy_bundle: EnemyBundle::new(transform, enemies_resources),
            health: Health { health },
            fridge: Fridge { attached_weapon },
            disabled: FridgeDisabled,

            level_object: LevelObject,
        }
    }
}

pub fn spawn_fridge(
    enemies_resources: &EnemiesResources,
    weapons_resources: &WeaponsResources,
    commands: &mut Commands,
    transform: Transform,
) {
    let weapon_transform = Transform::from_translation(FRIDGE_WEAPON_OFFSET).with_rotation(
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    let weapon = commands
        .spawn((
            PistolBundle::new(weapon_transform, weapons_resources),
            FridgeWeapon,
        ))
        .id();

    commands
        .spawn((FridgeBuldle::new(
            FRIDGE_HEALTH,
            Some(weapon),
            transform,
            enemies_resources,
        ),))
        .add_child(weapon);
}

fn fridge_enable(
    fridges: Query<Entity, With<FridgeDisabled>>,
    mut commands: Commands,
    mut level_started_events: EventReader<LevelStarted>,
) {
    for _ in level_started_events.read() {
        for fridge in fridges.iter() {
            commands
                .get_entity(fridge)
                .unwrap()
                .remove::<FridgeDisabled>();
        }
    }
}

#[allow(clippy::complexity)]
fn fridge_move(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Fridge>)>,
    mut fridges: Query<
        (&mut Transform, &mut KinematicCharacterController),
        (With<Fridge>, Without<FridgeDisabled>, Without<Player>),
    >,
) {
    let Ok(player_transfomr) = player.get_single() else {
        return;
    };

    for (mut fridge_transform, mut fridge_controller) in fridges.iter_mut() {
        let v = player_transfomr.translation - fridge_transform.translation;
        let direction = v.normalize();
        if FRIDGE_MIN_DISTANCE < v.length_squared() {
            let movement = direction * FRIDGE_SPEED * time.delta_seconds();
            fridge_controller.translation = Some(movement);
        }

        let enemy_forward = fridge_transform.rotation * Vec3::X;
        let mut angle = direction.angle_between(enemy_forward);
        let cross = direction.cross(enemy_forward);
        if 0.0 <= cross.z {
            angle *= -1.0;
        }
        fridge_transform.rotate_z(angle * time.delta_seconds());
    }
}

fn fridge_shoot(
    enemy_weapons: Query<(Entity, &GlobalTransform, &WeaponAttackTimer), With<FridgeWeapon>>,
    mut shoot_event: EventWriter<ShootEvent>,
) {
    for (weapon_entity, weapon_global_transform, weapon_attack_timer) in enemy_weapons.iter() {
        if weapon_attack_timer.attack_timer.finished() {
            shoot_event.send(ShootEvent {
                weapon_entity,
                weapon_translation: weapon_global_transform.translation(),
                direction: weapon_global_transform.back(),
            });
        }
    }
}

fn fridge_die(
    enemies_resources: Res<EnemiesResources>,
    fridges: Query<(Entity, &Transform, &Fridge), Without<FridgeWeapon>>,
    mut weapons: Query<(Entity, &mut Transform), With<FridgeWeapon>>,
    mut commands: Commands,
    mut kill_events: EventReader<KillEvent>,
) {
    for kill_event in kill_events.read() {
        if let Ok((fridge_entity, fridge_transform, fridge)) = fridges.get(kill_event.entity) {
            // spawn parts
            for x in 0..FRIDGE_PARTS_X {
                for y in 0..FRIDGE_PARTS_Y {
                    for z in 0..FRIDGE_PARTS_Z {
                        let x_pos = -(FRIDGE_DIMENTION_X + FRIDGE_DEATH_GAP_X) / 2.0
                            + (FRIDGE_PART_DIMENTION_X + FRIDGE_DEATH_GAP_DELTA_X) * x as f32;
                        let y_pos = -(FRIDGE_DIMENTION_Y + FRIDGE_DEATH_GAP_Y) / 2.0
                            + (FRIDGE_PART_DIMENTION_Y + FRIDGE_DEATH_GAP_DELTA_Y) * y as f32;
                        let z_pos = -(FRIDGE_DIMENTION_Z + FRIDGE_DEATH_GAP_Z) / 2.0
                            + (FRIDGE_PART_DIMENTION_Z + FRIDGE_DEATH_GAP_DELTA_Z) * z as f32
                            // to make all parts be above ground
                            + FRIDGE_DIMENTION_Z / 2.0;
                        let pos = Vec3::new(x_pos, y_pos, z_pos);
                        let translation = fridge_transform.transform_point(pos);
                        let transform = Transform::from_translation(translation)
                            .with_rotation(fridge_transform.rotation);
                        let linvel = (translation - fridge_transform.translation).normalize()
                            * FRIDGE_DEATH_PULSE_STENGTH;
                        commands.spawn((
                            PbrBundle {
                                mesh: enemies_resources.fridge_part_mesh.clone(),
                                material: enemies_resources.fridge_material.clone(),
                                transform,
                                ..default()
                            },
                            Collider::cuboid(
                                FRIDGE_PART_DIMENTION_X / 2.0,
                                FRIDGE_PART_DIMENTION_Y / 2.0,
                                FRIDGE_PART_DIMENTION_Z / 2.0,
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

            // drop weapon
            if let Some(attached_weapon) = fridge.attached_weapon {
                if let Ok((weapon, mut weapon_transform)) = weapons.get_mut(attached_weapon) {
                    commands
                        .get_entity(fridge_entity)
                        .unwrap()
                        .remove_children(&[weapon]);
                    *weapon_transform = *fridge_transform;
                    commands
                        .get_entity(weapon)
                        .unwrap()
                        .remove::<FridgeWeapon>()
                        .insert(FreeFloatingWeaponBundle::new(fridge_transform.translation));
                }
            }

            commands.get_entity(fridge_entity).unwrap().despawn();
        }
    }
}
