use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::level::LevelObject;

const PROJECTILE_VELOCITY: f32 = 10.0;

const PISTOL_PROJECTILE_OFFSET_SCALE: f32 = 2.0;
const PISTOL_PROJECTILE_SIZE: f32 = 0.3;
const PISTOL_AMMO: u32 = 10;
const PISTOL_DAMAGE: u32 = 10;
const PISTOL_ATTACK_SPEED: f32 = 1.0 / 4.0;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShootEvent>();
        app.add_systems(
            Update,
            (
                update_attack_timers,
                update_projectiles,
                shoot_pistol,
                // display_events,
            ),
        );
    }
}

#[derive(Component)]
struct ProjectileDirection {
    direction: Vec3,
}

#[derive(Component)]
struct Projectile {
    damage: u32,
}

#[derive(Event)]
pub struct ShootEvent {
    pub weapon_entity: Entity,
    pub weapon_translation: Vec3,
    pub direction: Vec3,
}

#[derive(Component)]
pub struct WeaponAttackTimer {
    pub attack_timer: Timer,
}

#[derive(Component)]
pub struct Pistol {
    pub ammo: u32,
}

fn update_attack_timers(time: Res<Time>, mut timers: Query<&mut WeaponAttackTimer>) {
    for mut timer in timers.iter_mut() {
        timer.attack_timer.tick(time.delta());
    }
}

fn shoot_pistol(
    pistols: Query<&Pistol>,
    mut commands: Commands,
    mut shoot_event: EventReader<ShootEvent>,
) {
    for e in shoot_event.read() {
        if pistols.get(e.weapon_entity).is_ok() {
            let translation = e.weapon_translation + e.direction * PISTOL_PROJECTILE_OFFSET_SCALE;
            // spawn projectiles
            commands
                .spawn((
                    TransformBundle::from_transform(Transform::from_translation(translation)),
                    RigidBody::KinematicVelocityBased,
                    Collider::ball(PISTOL_PROJECTILE_SIZE),
                    Velocity {
                        linvel: e.direction * PROJECTILE_VELOCITY,
                        ..default()
                    },
                    Projectile {
                        damage: PISTOL_DAMAGE,
                    },
                ));
            // start shooting animation
        }
    }
}
fn animate_pistol_shoot(pistols: Query<&Pistol>) {}
fn animate_pistol_in_the_air(pistols: Query<&Pistol>) {}

fn update_projectiles(
    rapier_context: Res<RapierContext>,
    projectiles: Query<Entity, With<Projectile>>,
    level_objects: Query<Entity, With<LevelObject>>,
    mut commands: Commands,
) {
    for projectile in projectiles.iter() {
        for contact_pair in rapier_context.contacts_with(projectile) {
            if level_objects
                .get(contact_pair.collider1())
                .or(level_objects.get(contact_pair.collider2()))
                .is_ok()
            {
                commands.get_entity(projectile).unwrap().despawn();
            }
        }
    }
}

fn display_events(
    rapier_context: Res<RapierContext>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for p in rapier_context.contact_pairs() {
        println!("pair: {:?} : {:?}", p.collider1(), p.collider2());
    }
    for collision_event in collision_events.read() {
        println!("Received collision event: {:?}", collision_event);
    }
}
