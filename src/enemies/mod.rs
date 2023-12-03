use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    player::Player,
    weapons::{pistol::PistolBundle, ShootEvent, Weapon, WeaponsResources},
};

const ENEMY_SPEED: f32 = 5.0;
const ENEMY_MIN_DISTANCE: f32 = 200.0;
const ENEMY_WEAPON_OFFSET: Vec3 = Vec3::new(1.0, -1.0, 0.5);

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_resources);
        app.add_systems(PostStartup, spawn);
        app.add_systems(Update, (enemy_move, enemy_shoot));
    }
}

#[derive(Resource)]
pub struct EnemiesResources {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyWeapon;

#[derive(Bundle)]
pub struct EnemyBundle {
    pbr: PbrBundle,
    collider: Collider,
    rigid_body: RigidBody,
    velocity: Velocity,
    enemy: Enemy,
}

impl EnemyBundle {
    pub fn new(transform: Transform, enemies_resources: &EnemiesResources) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: enemies_resources.mesh.clone(),
                material: enemies_resources.material.clone(),
                transform,
                ..default()
            },
            collider: Collider::capsule(Vec3::new(0.0, 0.0, -3.5), Vec3::new(0.0, 0.0, 3.5), 2.0),
            rigid_body: RigidBody::KinematicVelocityBased,
            velocity: Velocity::default(),
            enemy: Enemy,
        }
    }
}

fn init_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // forward = -Z
    let mesh = meshes.add(shape::Box::new(0.5, 3.5, 7.0).into());
    let material = materials.add(Color::RED.into());

    commands.insert_resource(EnemiesResources { mesh, material });
}

fn spawn(
    enemies_resources: Res<EnemiesResources>,
    weapons_resources: Res<WeaponsResources>,
    mut commands: Commands,
) {
    let translation = Vec3::new(20.0, 0.0, 5.0);
    let transform = Transform::from_translation(translation);
    let weapon_transform = Transform::from_translation(ENEMY_WEAPON_OFFSET).with_rotation(
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    commands
        .spawn((EnemyBundle::new(transform, enemies_resources.as_ref()),))
        .with_children(|builder| {
            builder.spawn((
                PistolBundle::new(weapon_transform, weapons_resources.as_ref()),
                EnemyWeapon,
            ));
        });
}

#[allow(clippy::complexity)]
fn enemy_move(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(&mut Velocity, &mut Transform), (With<Enemy>, Without<Player>)>,
) {
    let Ok(player_transfomr) = player.get_single() else {
        return;
    };

    for (mut enemy_velocity, mut enemy_transform) in enemies.iter_mut() {
        let v = player_transfomr.translation - enemy_transform.translation;
        let direction = v.normalize();
        if v.length_squared() < ENEMY_MIN_DISTANCE {
            enemy_velocity.linvel = Vec3::ZERO;
        } else {
            enemy_velocity.linvel = direction * ENEMY_SPEED;
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

fn enemy_shoot(
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
