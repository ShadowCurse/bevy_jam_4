use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::{FreeFloatingWeapon, Projectile, ShootEvent, Weapon, WeaponsResources};

const PISTOL_PROJECTILE_VELOCITY: f32 = 10.0;
const PISTOL_PROJECTILE_OFFSET_SCALE: f32 = 2.0;
const PISTOL_PROJECTILE_SIZE: f32 = 0.3;
const PISTOL_AMMO: u32 = 10;
const PISTOL_DAMAGE: u32 = 10;
const PISTOL_ATTACK_SPEED: f32 = 1.0 / 4.0;

pub struct PistolPlugin;

impl Plugin for PistolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn,));
        app.add_systems(Update, (shoot_pistol,));
    }
}

#[derive(Component)]
pub struct Pistol {
    pub ammo: u32,
}

#[derive(Bundle)]
pub struct PistolBundle {
    pbr: PbrBundle,
    collider: Collider,
    pistol: Pistol,
    weapon: Weapon,
}

impl PistolBundle {
    pub fn new(transform: Transform, weapons_resources: &WeaponsResources) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: weapons_resources.pistol_mesh.clone(),
                material: weapons_resources.pistol_material.clone(),
                transform,
                ..default()
            },
            collider: Collider::ball(0.6),
            pistol: Pistol { ammo: PISTOL_AMMO },
            weapon: Weapon::new(PISTOL_ATTACK_SPEED),
        }
    }
}

fn spawn(weapons_resources: Res<WeaponsResources>, mut commands: Commands) {
    let translation = Vec3::new(10.0, 10.0, 5.0);
    let transform = Transform::from_translation(translation)
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PistolBundle::new(transform, weapons_resources.as_ref()),
        FreeFloatingWeapon {
            original_translation: translation,
        },
    ));
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
            commands.spawn((
                TransformBundle::from_transform(Transform::from_translation(translation)),
                RigidBody::KinematicVelocityBased,
                Collider::ball(PISTOL_PROJECTILE_SIZE),
                Velocity {
                    linvel: e.direction * PISTOL_PROJECTILE_VELOCITY,
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
