use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{damage::Damage, GlobalState};

use super::{
    Ammo, ShellBundle, FreeFloatingWeaponBundle, ProjectileBundle, ShootEvent, WeaponAttackTimer,
    WeaponShootAnimation, WeaponsAssets, WeaponsResources,
};

const PISTOL_AMMO: u32 = 10;
const PISTOL_DAMAGE: i32 = 10;
const PISTOL_ATTACK_SPEED: f32 = 1.0 / 4.0;
// const PISTOL_ATTACK_SPEED: f32 = 1.0;
const PISTOL_PROJECTILE_VELOCITY: f32 = 500.0;
const PISTOL_PROJECTILE_OFFSET_SCALE: f32 = 2.0;

// Needs to be bigger that (1 / attack_speed) * 2
// because animatino played for 2 directions
const PISTOL_ANIMATION_SPEED: f32 = 10.0;
const PISTOL_ANIMATION_TARGET_OFFSET: Vec3 = Vec3::new(0.2, 0.2, 0.0);
const PISTOL_ANIMATION_TARGET_ROTATION_X: f32 = std::f32::consts::FRAC_PI_8;

const PISTOL_CLIP_INITIAL_VELOCITY: f32 = 10.0;

pub struct PistolPlugin;

impl Plugin for PistolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (shoot_pistol,).run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Pistol;

#[derive(Component)]
pub struct PistolModel;

#[derive(Bundle)]
pub struct PistolBundle {
    pub transform_bundle: TransformBundle,
    pub inherited_visibility: InheritedVisibility,
    pub pistol: Pistol,
    pub ammo: Ammo,
    pub weapon: WeaponAttackTimer,
}

impl Default for PistolBundle {
    fn default() -> Self {
        Self {
            transform_bundle: TransformBundle::default(),
            inherited_visibility: InheritedVisibility::VISIBLE,
            pistol: Pistol,
            ammo: Ammo { ammo: PISTOL_AMMO },
            weapon: WeaponAttackTimer::new(PISTOL_ATTACK_SPEED),
        }
    }
}

pub fn spawn_pistol(weapons_assets: &WeaponsAssets, commands: &mut Commands, transform: Transform) {
    // let transform = transform.with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands
        .spawn((
            PistolBundle {
                transform_bundle: TransformBundle::from_transform(transform),
                ..default()
            },
            FreeFloatingWeaponBundle::new(transform.translation),
        ))
        .with_children(|builder| {
            builder.spawn((
                SceneBundle {
                    scene: weapons_assets.pistol_scene.clone(),
                    // transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)),
                    ..default()
                },
                PistolModel,
            ));
        });
}

fn shoot_pistol(
    pistols: Query<&Children, With<Pistol>>,
    weapons_assets: Res<WeaponsAssets>,
    weapons_resources: Res<WeaponsResources>,
    mut commands: Commands,
    mut shoot_event: EventReader<ShootEvent>,
) {
    for e in shoot_event.read() {
        if let Ok(pistol_children) = pistols.get(e.weapon_entity) {
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

            // spawn shell
            let clip_direction = e.direction.cross(Vec3::Z);
            commands.spawn(ShellBundle {
                scene_bundle: SceneBundle {
                    scene: weapons_assets.pistol_shell_scene.clone(),
                    transform: Transform::from_translation(e.weapon_translation)
                        .with_scale(Vec3::new(2.0, 2.0, 2.0)),
                    ..default()
                },
                velocity: Velocity {
                    linvel: clip_direction * PISTOL_CLIP_INITIAL_VELOCITY,
                    ..default()
                },
                ..default()
            });

            // start shooting animation
            let pistol_model = pistol_children[0];
            let Some(mut e) = commands.get_entity(pistol_model) else {
                continue;
            };
            e.insert(WeaponShootAnimation {
                animation_speed: PISTOL_ANIMATION_SPEED,
                animate_forward: true,
                progress: 0.0,
                initial_transform: Transform::default(),
                target_transform: Transform::from_translation(PISTOL_ANIMATION_TARGET_OFFSET)
                    .with_rotation(Quat::from_rotation_x(PISTOL_ANIMATION_TARGET_ROTATION_X)),
            });
        }
    }
}
