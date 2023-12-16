use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{level::LevelObject, GlobalState, COLLISION_GROUP_PICKUP, COLLISION_GROUP_PLAYER};

const COLLIDER_RADIUS: f32 = 1.5;
const ROTATION_SPEED: f32 = 0.4;
const AMPLITUDE_MODIFIER: f32 = 0.5;
const BOUNCE_SPEED_MODIFIER: f32 = 2.0;

pub struct FloatingPlugin;

impl Plugin for FloatingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_floating_objects.run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct FloatingObject {
    pub original_translation: Vec3,
}

#[derive(Bundle)]
pub struct FloatingObjectBundle {
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub floating_object: FloatingObject,

    pub level_object: LevelObject,
}

impl FloatingObjectBundle {
    pub fn new(original_translation: Vec3) -> Self {
        Self {
            collider: Collider::ball(COLLIDER_RADIUS),
            collision_groups: CollisionGroups::new(COLLISION_GROUP_PICKUP, COLLISION_GROUP_PLAYER),
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            floating_object: FloatingObject {
                original_translation,
            },

            level_object: LevelObject,
        }
    }
}

fn update_floating_objects(time: Res<Time>, mut weapons: Query<(&FloatingObject, &mut Transform)>) {
    for (floating, mut weapon_transform) in weapons.iter_mut() {
        weapon_transform.translation = floating.original_translation
            + Vec3::NEG_Z
                * AMPLITUDE_MODIFIER
                * (time.elapsed().as_secs_f32() * BOUNCE_SPEED_MODIFIER).sin();
        weapon_transform.rotate_z(time.delta_seconds() * ROTATION_SPEED);
    }
}
