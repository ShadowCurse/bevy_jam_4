use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};

use crate::{
    player::Player, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_PROJECTILES,
};

use super::{LevelCollider, LevelFinished, LevelObject, LevelResources, LevelSwitch};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (level_finished, portal_use));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalType {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct Portal {
    pub portal_type: PortalType,
    pub grid_pox: usize,
}

#[derive(Bundle)]
pub struct PortalBundle {
    pub pbr_bundle: PbrBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub rigid_body: RigidBody,
    pub active_events: ActiveEvents,
    pub portal: Portal,
    pub level_collider: LevelCollider,

    pub level_object: LevelObject,
}

impl Default for PortalBundle {
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
            portal: Portal {
                portal_type: PortalType::Top,
                grid_pox: 0,
            },
            level_collider: LevelCollider,

            level_object: LevelObject,
        }
    }
}

impl PortalBundle {
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        collider: Collider,
        portal: Portal,
    ) -> Self {
        Self {
            pbr_bundle: PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            collider,
            portal,
            ..default()
        }
    }
}

fn level_finished(
    level_resources: Res<LevelResources>,
    mut commands: Commands,
    mut level_finished_events: EventReader<LevelFinished>,
    mut portals: Query<(Entity, &mut Handle<StandardMaterial>), With<Portal>>,
) {
    if !level_finished_events.is_empty() {
        level_finished_events.clear();
        for (portal, mut portal_material) in portals.iter_mut() {
            *portal_material = level_resources.portal_open_material.clone();
            commands
                .get_entity(portal)
                .unwrap()
                .insert(Sensor)
                .remove::<RigidBody>();
        }
    }
}

fn portal_use(
    player: Query<Entity, With<Player>>,
    portals: Query<&Portal>,
    mut level_switch_events: EventWriter<LevelSwitch>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    let Ok(player) = player.get_single() else {
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
        let portal = if collider_1 == &player {
            if let Ok(p) = portals.get(*collider_2) {
                p
            } else {
                continue;
            }
        } else if collider_2 == &player {
            if let Ok(p) = portals.get(*collider_1) {
                p
            } else {
                continue;
            }
        } else {
            continue;
        };

        level_switch_events.send(LevelSwitch {
            exit_portal: *portal,
        });
    }
}
