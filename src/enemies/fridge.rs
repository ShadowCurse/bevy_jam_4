use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    player::Player,
    weapons::{pistol::PistolBundle, ShootEvent, Weapon, WeaponsResources},
};

use super::{EnemiesResources, EnemyBundle, EnemyWeapon};

const FRIDGE_SPEED: f32 = 5.0;
const FRIDGE_MIN_DISTANCE: f32 = 200.0;
const FRIDGE_WEAPON_OFFSET: Vec3 = Vec3::new(1.0, -1.0, 0.5);

pub struct FridgePlugin;

impl Plugin for FridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn);
        app.add_systems(Update, (fridge_move, fridge_shoot));
    }
}

#[derive(Component)]
pub struct Fridge;

fn spawn(
    enemies_resources: Res<EnemiesResources>,
    weapons_resources: Res<WeaponsResources>,
    mut commands: Commands,
) {
    let translation = Vec3::new(20.0, 0.0, 5.0);
    let transform = Transform::from_translation(translation);
    let weapon_transform = Transform::from_translation(FRIDGE_WEAPON_OFFSET).with_rotation(
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    commands
        .spawn((
            EnemyBundle::new(transform, enemies_resources.as_ref()),
            Fridge,
        ))
        .with_children(|builder| {
            builder.spawn((
                PistolBundle::new(weapon_transform, weapons_resources.as_ref()),
                EnemyWeapon,
            ));
        });
}

#[allow(clippy::complexity)]
fn fridge_move(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Fridge>)>,
    mut enemies: Query<(&mut Velocity, &mut Transform), (With<Fridge>, Without<Player>)>,
) {
    let Ok(player_transfomr) = player.get_single() else {
        return;
    };

    for (mut enemy_velocity, mut enemy_transform) in enemies.iter_mut() {
        let v = player_transfomr.translation - enemy_transform.translation;
        let direction = v.normalize();
        if v.length_squared() < FRIDGE_MIN_DISTANCE {
            enemy_velocity.linvel = Vec3::ZERO;
        } else {
            enemy_velocity.linvel = direction * FRIDGE_SPEED;
        }

        let enemy_forward = enemy_transform.rotation * Vec3::X;
        let mut angle = direction.angle_between(enemy_forward);
        let cross = direction.cross(enemy_forward);
        if 0.0 <= cross.z {
            angle *= -1.0;
        }
        enemy_transform.rotate_z(angle * time.delta_seconds());
    }
}

fn fridge_shoot(
    enemy_weapons: Query<(Entity, &GlobalTransform, &Weapon), With<EnemyWeapon>>,
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
