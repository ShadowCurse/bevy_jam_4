use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};

use crate::{enemies::Enemy, weapons::Projectile};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<KillEvent>();
        app.add_systems(Update, apply_damage);
    }
}

#[derive(Event)]
pub struct KillEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct Damage {
    pub damage: i32,
}

#[derive(Component)]
pub struct Health {
    pub health: i32,
}

fn apply_damage(
    projectiles: Query<&Damage, With<Projectile>>,
    mut commands: Commands,
    mut kill_events: EventWriter<KillEvent>,
    mut collision_events: EventReader<CollisionEvent>,
    mut enemies: Query<(Entity, &mut Health), With<Enemy>>,
) {
    for collision_event in collision_events.read() {
        let (collider_1, collider_2, flags) = match collision_event {
            CollisionEvent::Started(c1, c2, f) => (c1, c2, f),
            CollisionEvent::Stopped(c1, c2, f) => (c1, c2, f),
        };
        if flags.contains(CollisionEventFlags::REMOVED) {
            return;
        }

        let (projectile_damage, (enemy, mut enemy_health)) =
            if let Ok(p) = projectiles.get(*collider_1) {
                let e = if let Ok(e) = enemies.get_mut(*collider_2) {
                    e
                } else {
                    continue;
                };
                (p, e)
            } else if let Ok(p) = projectiles.get(*collider_2) {
                let e = if let Ok(e) = enemies.get_mut(*collider_1) {
                    e
                } else {
                    continue;
                };
                (p, e)
            } else {
                continue;
            };

        enemy_health.health -= projectile_damage.damage;
        if enemy_health.health <= 0 {
            commands.get_entity(enemy).unwrap().remove::<Health>();
            kill_events.send(KillEvent { entity: enemy });
        }
    }
}
