use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::damage::Damage;

use super::{
    FreeFloatingWeapon, FreeFloatingWeaponBundle, ProjectileBundle, ShootEvent, WeaponAttackTimer,
    WeaponsResources,
};

const PISTOL_AMMO: u32 = 10;
const PISTOL_DAMAGE: i32 = 10;
const PISTOL_ATTACK_SPEED: f32 = 1.0 / 4.0;
const PISTOL_PROJECTILE_VELOCITY: f32 = 500.0;
const PISTOL_PROJECTILE_OFFSET_SCALE: f32 = 2.0;

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
    pistol: Pistol,
    weapon: WeaponAttackTimer,
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
            pistol: Pistol { ammo: PISTOL_AMMO },
            weapon: WeaponAttackTimer::new(PISTOL_ATTACK_SPEED),
        }
    }
}

fn spawn(weapons_resources: Res<WeaponsResources>, mut commands: Commands) {
    let translation = Vec3::new(10.0, 10.0, 5.0);
    let transform = Transform::from_translation(translation)
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PistolBundle::new(transform, weapons_resources.as_ref()),
        FreeFloatingWeaponBundle::new(translation),
    ));

    let translation = Vec3::new(10.0, 30.0, 5.0);
    let transform = Transform::from_translation(translation)
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PistolBundle::new(transform, weapons_resources.as_ref()),
        FreeFloatingWeaponBundle::new(translation),
    ));
}

fn shoot_pistol(
    pistols: Query<&Pistol>,
    weapons_resources: Res<WeaponsResources>,
    mut commands: Commands,
    mut shoot_event: EventReader<ShootEvent>,
) {
    for e in shoot_event.read() {
        if pistols.get(e.weapon_entity).is_ok() {
            let translation = e.weapon_translation + e.direction * PISTOL_PROJECTILE_OFFSET_SCALE;
            // spawn projectiles
            commands.spawn(ProjectileBundle {
                pbr_bundle: PbrBundle {
                    mesh: weapons_resources.projectile_mesh.clone(),
                    material: weapons_resources.projectile_material.clone(),
                    transform: Transform::from_translation(translation),
                    ..default()
                },
                velocity: Velocity {
                    linvel: e.direction * PISTOL_PROJECTILE_VELOCITY,
                    ..default()
                },
                damage: Damage {
                    damage: PISTOL_DAMAGE,
                },
                ..default()
            });
            // start shooting animation
        }
    }
}
