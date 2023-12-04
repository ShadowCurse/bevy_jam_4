use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
    rapier_context: Res<RapierContext>,
    projectiles: Query<(Entity, &Damage), With<Projectile>>,
    mut commands: Commands,
    mut kill_events: EventWriter<KillEvent>,
    mut enemies: Query<(Entity, &mut Health), With<Enemy>>,
) {
    for (projectile, projectile_damage) in projectiles.iter() {
        for contact_pair in rapier_context.contacts_with(projectile) {
            let (enemy, mut enemy_health) = if let Ok(e) = enemies.get_mut(contact_pair.collider1())
            {
                e
            } else if let Ok(e) = enemies.get_mut(contact_pair.collider2()) {
                e
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
}
