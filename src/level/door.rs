use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};

use crate::{
    player::Player, GameState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL,
    COLLISION_GROUP_PLAYER, COLLISION_GROUP_PROJECTILES,
};

use super::{
    LevelCollider, LevelFinished, LevelObject, LevelResources, LevelStarted, LevelSwitch,
    COLUMN_HIGHT, COLUMN_SIZE, DOOR_THICKNESS,
};

const DOOR_ANIMATION_DISTANCE: f32 = COLUMN_SIZE - 0.2;
const DOOR_ANIMATION_SPEED: f32 = 2.0;

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DoorAnimationFinished>();

        app.add_systems(
            Update,
            (level_finished, door_use, animate_door).run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorType {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Locked,
    Unlocked,
    Used,
    TemporaryOpen,
}

#[derive(Event)]
pub struct DoorAnimationFinished {
    pub animation_type: DoorAnimationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorAnimationType {
    Open,
    Close,
}

#[derive(Component)]
pub struct DoorAnimation {
    animation_type: DoorAnimationType,
    animation_progress: f32,
    original_translation: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct Door {
    pub door_type: DoorType,
    pub door_state: DoorState,
    pub grid_pos: usize,
}

#[derive(Bundle)]
pub struct DoorBundle {
    pub pbr_bundle: PbrBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub rigid_body: RigidBody,
    pub active_events: ActiveEvents,
    pub door: Door,
    pub level_collider: LevelCollider,

    pub level_object: LevelObject,
}

impl Default for DoorBundle {
    fn default() -> Self {
        Self {
            pbr_bundle: PbrBundle::default(),
            collider: Collider::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_LEVEL,
                COLLISION_GROUP_ENEMY | COLLISION_GROUP_PLAYER | COLLISION_GROUP_PROJECTILES,
            ),
            rigid_body: RigidBody::Fixed,
            active_events: ActiveEvents::COLLISION_EVENTS,
            door: Door {
                door_type: DoorType::Top,
                door_state: DoorState::Locked,
                grid_pos: 0,
            },
            level_collider: LevelCollider,

            level_object: LevelObject,
        }
    }
}

impl DoorBundle {
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        collider: Collider,
        door: Door,
    ) -> Self {
        Self {
            pbr_bundle: PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            collider,
            door,
            ..default()
        }
    }
}

#[derive(Component)]
pub struct DoorSensor {
    associated_door: Entity,
}

#[derive(Bundle)]
pub struct DoorSensorBundle {
    pub transform_bundle: TransformBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub door_sensor: DoorSensor,

    pub level_object: LevelObject,
}

impl Default for DoorSensorBundle {
    fn default() -> Self {
        Self {
            transform_bundle: TransformBundle::default(),
            collider: Collider::default(),
            collision_groups: CollisionGroups::new(COLLISION_GROUP_LEVEL, COLLISION_GROUP_PLAYER),
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            door_sensor: DoorSensor {
                associated_door: Entity::PLACEHOLDER,
            },

            level_object: LevelObject,
        }
    }
}

pub fn spawn_door(
    level_resources: &LevelResources,
    commands: &mut Commands,
    transform: Transform,
    door: Door,
) {
    let transform = match door.door_type {
        DoorType::Top | DoorType::Bottom => transform,
        DoorType::Left | DoorType::Right => {
            transform.with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        }
    };

    let door_collider =
        Collider::cuboid(COLUMN_SIZE / 2.0, DOOR_THICKNESS / 2.0, COLUMN_HIGHT / 2.0);
    let door_entity = if door.door_state == DoorState::TemporaryOpen {
        commands
            .spawn((
                DoorBundle::new(
                    level_resources.door_mesh.clone(),
                    level_resources.door_closed_material.clone(),
                    Transform::default(),
                    door_collider,
                    door,
                ),
                DoorAnimation {
                    animation_type: DoorAnimationType::Open,
                    animation_progress: 0.0,
                    original_translation: Vec3::default(),
                },
            ))
            .id()
    } else {
        commands
            .spawn(DoorBundle::new(
                level_resources.door_mesh.clone(),
                level_resources.door_closed_material.clone(),
                Transform::default(),
                door_collider,
                door,
            ))
            .id()
    };

    let sensor_collider =
        Collider::cuboid(COLUMN_SIZE / 2.0, COLUMN_SIZE / 2.0, COLUMN_HIGHT / 2.0);
    commands
        .spawn(DoorSensorBundle {
            transform_bundle: TransformBundle::from_transform(transform),
            collider: sensor_collider,
            door_sensor: DoorSensor {
                associated_door: door_entity,
            },
            ..default()
        })
        .add_child(door_entity);
}

fn level_finished(
    level_resources: Res<LevelResources>,
    mut level_finished_events: EventReader<LevelFinished>,
    mut doors: Query<(&mut Door, &mut Handle<StandardMaterial>), With<Door>>,
) {
    if !level_finished_events.is_empty() {
        level_finished_events.clear();
        for (mut door, mut door_material) in doors.iter_mut() {
            *door_material = level_resources.door_open_material.clone();
            door.door_state = DoorState::Unlocked;
        }
    }
}

fn door_use(
    player: Query<(Entity, &Transform), With<Player>>,
    door_sensors: Query<(&DoorSensor, &Transform), Without<Player>>,
    mut commands: Commands,
    mut doors: Query<(Entity, &Transform, &mut Door), Without<Player>>,
    mut level_start_events: EventWriter<LevelStarted>,
    mut level_switch_events: EventWriter<LevelSwitch>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    let Ok((player, player_transform)) = player.get_single() else {
        return;
    };

    for collision_event in collision_events.read() {
        let (collider_1, collider_2, flags) = match collision_event {
            CollisionEvent::Started(c1, c2, f) => (c1, c2, f),
            CollisionEvent::Stopped(c1, c2, f) => (c1, c2, f),
        };

        if flags.contains(CollisionEventFlags::REMOVED)
            || !flags.contains(CollisionEventFlags::SENSOR)
        {
            return;
        }
        let (door_sensor, door_sensor_transform) = if collider_1 == &player {
            if let Ok(p) = door_sensors.get(*collider_2) {
                p
            } else {
                continue;
            }
        } else if collider_2 == &player {
            if let Ok(p) = door_sensors.get(*collider_1) {
                p
            } else {
                continue;
            }
        } else {
            continue;
        };

        let (door_entity, door_transform, mut door) =
            doors.get_mut(door_sensor.associated_door).unwrap();

        if door.door_state == DoorState::TemporaryOpen {
            match collision_event {
                CollisionEvent::Started(_, _, _) => return,
                CollisionEvent::Stopped(_, _, _) => {
                    // Player enters the level with this door
                    let player_went_though = match door.door_type {
                        DoorType::Top => {
                            player_transform.translation.y < door_sensor_transform.translation.y
                        }
                        DoorType::Bottom => {
                            player_transform.translation.y > door_sensor_transform.translation.y
                        }
                        DoorType::Left => {
                            player_transform.translation.x > door_sensor_transform.translation.x
                        }
                        DoorType::Right => {
                            player_transform.translation.x < door_sensor_transform.translation.x
                        }
                    };
                    if player_went_though {
                        level_start_events.send(LevelStarted);

                        door.door_state = DoorState::Locked;
                        commands
                            .get_entity(door_entity)
                            .unwrap()
                            .insert(DoorAnimation {
                                animation_type: DoorAnimationType::Close,
                                animation_progress: 0.0,
                                original_translation: door_transform.translation,
                            });
                    }
                }
            };
        }

        if door.door_state != DoorState::Unlocked {
            return;
        }

        door.door_state = DoorState::Used;

        println!("used door {door:?}");
        level_switch_events.send(LevelSwitch { exit_door: *door });

        commands
            .get_entity(door_entity)
            .unwrap()
            .insert(DoorAnimation {
                animation_type: DoorAnimationType::Open,
                animation_progress: 0.0,
                original_translation: door_transform.translation,
            });
    }
}

fn animate_door(
    time: Res<Time>,
    mut commands: Commands,
    mut door_amimation_finished_events: EventWriter<DoorAnimationFinished>,
    mut animated_doors: Query<(Entity, &mut DoorAnimation, &mut Transform)>,
) {
    for (door_entity, mut animation, mut transform) in animated_doors.iter_mut() {
        let target_translation = match animation.animation_type {
            DoorAnimationType::Open => {
                animation.original_translation + Vec3::X * DOOR_ANIMATION_DISTANCE
            }
            DoorAnimationType::Close => {
                animation.original_translation - Vec3::X * DOOR_ANIMATION_DISTANCE
            }
        };

        animation.animation_progress += time.delta_seconds() * DOOR_ANIMATION_SPEED;

        transform.translation = animation
            .original_translation
            .lerp(target_translation, animation.animation_progress);

        if 1.0 <= animation.animation_progress {
            println!("sending animation finished event");
            door_amimation_finished_events.send(DoorAnimationFinished {
                animation_type: animation.animation_type,
            });

            commands
                .get_entity(door_entity)
                .unwrap()
                .remove::<DoorAnimation>();
        }
    }
}
