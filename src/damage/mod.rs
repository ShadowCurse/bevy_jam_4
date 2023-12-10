use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};

use crate::{weapons::Projectile, GlobalState};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>();
        app.add_event::<KillEvent>();

        app.add_systems(Update, apply_damage.run_if(in_state(GlobalState::InGame)));
    }
}

#[derive(Clone, Copy, Event)]
pub struct DamageEvent {
    pub entity: Entity,
    pub direction: Vec3,
}

#[derive(Clone, Copy, Event)]
pub struct KillEvent {
    pub entity: Entity,
}

#[derive(Default, Component)]
pub struct Damage {
    pub damage: i32,
}

#[derive(Default, Component)]
pub struct Health {
    pub health: i32,
}

fn apply_damage(
    projectiles: Query<&Projectile>,
    damage_objects: Query<(Entity, &Damage)>,
    mut commands: Commands,
    mut kill_events: EventWriter<KillEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    mut collision_events: EventReader<CollisionEvent>,
    mut entities: Query<(Entity, &mut Health)>,
) {
    for collision_event in collision_events.read() {
        let (collider_1, collider_2, flags) = match collision_event {
            CollisionEvent::Started(c1, c2, f) => (c1, c2, f),
            CollisionEvent::Stopped(c1, c2, f) => (c1, c2, f),
        };
        if flags.contains(CollisionEventFlags::SENSOR) {
            return;
        }
        println!("got collision event: {:?}", collision_event);

        let ((damage_entity, damage), (entity, mut entity_health)) =
            if let Ok(p) = damage_objects.get(*collider_1) {
                let e = if let Ok(e) = entities.get_mut(*collider_2) {
                    e
                } else {
                    continue;
                };
                (p, e)
            } else if let Ok(p) = damage_objects.get(*collider_2) {
                let e = if let Ok(e) = entities.get_mut(*collider_1) {
                    e
                } else {
                    continue;
                };
                (p, e)
            } else {
                continue;
            };

        // skip enemies that were killed by prevous iterations
        if entity_health.health <= 0 {
            continue;
        }
        entity_health.health -= damage.damage;

        let Some(mut e) = commands.get_entity(damage_entity) else {
            continue;
        };
        e.remove::<Damage>();

        if entity_health.health <= 0 {
            let Some(mut e) = commands.get_entity(entity) else {
                continue;
            };
            e.remove::<Health>();
            kill_events.send(KillEvent { entity });
        } else {
            let Ok(projectile) = projectiles.get(damage_entity) else {
                continue;
            };
            damage_events.send(DamageEvent {
                entity,
                direction: projectile.direction,
            });
        }
    }
}
