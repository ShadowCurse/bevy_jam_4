use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    damage::Damage, level::LevelObject, GlobalState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL,
    COLLISION_GROUP_PICKUP, COLLISION_GROUP_PLAYER, COLLISION_GROUP_PROJECTILES,
};

const DEFAULT_PROJECTILE_SIZE: f32 = 0.125;
const DEFAULT_CLIP_SIZE: f32 = 0.01;
const DEFAULT_CLIP_LENGTH: f32 = 0.02;

const FREE_FLOATING_WEAPON_COLLIDER_RADIUS: f32 = 1.5;
const FREE_FLOATING_WEAPON_ROTATION_SPEED: f32 = 0.4;
const FREE_FLOATING_WEAPON_AMPLITUDE_MODIFIER: f32 = 0.5;
const FREE_FLOATING_WEAPON_BOUNCE_SPEED_MODIFIER: f32 = 2.0;

// Pistol
const PISTOL_AMMO: u32 = 10;
const PISTOL_DAMAGE: i32 = 10;
const PISTOL_ATTACK_SPEED: f32 = 1.0 / 4.0;
const PISTOL_PROJECTILE_VELOCITY: f32 = 500.0;
const PISTOL_PROJECTILE_OFFSET_SCALE: f32 = 2.0;

// Needs to be bigger that (1 / attack_speed) * 2
// because animatino played for 2 directions
const PISTOL_ANIMATION_SPEED: f32 = 10.0;
const PISTOL_ANIMATION_FORWARD: bool = true;
const PISTOL_ANIMATION_BACKWARD: bool = true;
const PISTOL_ANIMATION_TARGET_OFFSET: Vec3 = Vec3::new(0.2, 0.2, 0.0);
const PISTOL_ANIMATION_TARGET_ROTATION_X: f32 = std::f32::consts::FRAC_PI_8;
const PISTOL_ANIMATION_TARGET_ROTATION_Y: f32 = 0.0;
const PISTOL_SHELL_INITIAL_VELOCITY: f32 = 10.0;

// Shotgun
const SHOTGUN_AMMO: u32 = 5;
const SHOTGUN_DAMAGE: i32 = 5;
const SHOTGUN_ATTACK_SPEED: f32 = 1.0 / 2.0;
const SHOTGUN_PROJECTILE_VELOCITY: f32 = 500.0;
const SHOTGUN_PROJECTILE_OFFSET_SCALE: f32 = 2.2;

// Needs to be bigger that (1 / attack_speed) * 2
// because animatino played for 2 directions
const SHOTGUN_ANIMATION_SPEED: f32 = 5.0;
const SHOTGUN_ANIMATION_FORWARD: bool = true;
const SHOTGUN_ANIMATION_BACKWARD: bool = true;
const SHOTGUN_ANIMATION_TARGET_OFFSET: Vec3 = Vec3::new(0.2, 0.2, 0.0);
const SHOTGUN_ANIMATION_TARGET_ROTATION_X: f32 = std::f32::consts::FRAC_PI_8;
const SHOTGUN_ANIMATION_TARGET_ROTATION_Y: f32 = 0.0;
const SHOTGUN_SHELL_INITIAL_VELOCITY: f32 = 10.0;

// Minigun
const MINIGUN_AMMO: u32 = 50;
const MINIGUN_DAMAGE: i32 = 10;
const MINIGUN_ATTACK_SPEED: f32 = 1.0 / 8.0;
const MINIGUN_PROJECTILE_VELOCITY: f32 = 500.0;
const MINIGUN_PROJECTILE_OFFSET_SCALE: f32 = 3.0;

// Needs to be bigger that (1 / attack_speed)
const MINIGUN_ANIMATION_SPEED: f32 = 9.0;
const MINIGUN_ANIMATION_FORWARD: bool = true;
const MINIGUN_ANIMATION_BACKWARD: bool = false;
const MINIGUN_ANIMATION_TARGET_OFFSET: Vec3 = Vec3::ZERO;
const MINIGUN_ANIMATION_TARGET_ROTATION_X: f32 = 0.0;
const MINIGUN_ANIMATION_TARGET_ROTATION_Y: f32 = std::f32::consts::FRAC_PI_2;
const MINIGUN_SHELL_INITIAL_VELOCITY: f32 = 10.0;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, WeaponAssets>(GlobalState::AssetLoading);

        app.add_event::<ShootEvent>();

        app.add_systems(
            OnTransition {
                from: GlobalState::AssetLoading,
                to: GlobalState::MainMenu,
            },
            init_resources,
        );
        app.add_systems(
            Update,
            (
                update_attack_timers,
                update_free_floating_weapons,
                weapon_shoot,
                weapon_animation,
                // display_events,
            )
                .run_if(in_state(GlobalState::InGame)),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct WeaponAssets {
    #[asset(path = "pistol/pistol.glb#Scene0")]
    pub pistol_scene: Handle<Scene>,
    #[asset(path = "pistol/pistol_shell.glb#Scene0")]
    pub pistol_shell_scene: Handle<Scene>,
    #[asset(path = "shotgun/shotgun.glb#Scene0")]
    pub shotgun_scene: Handle<Scene>,
    #[asset(path = "shotgun/shotgun_shell.glb#Scene0")]
    pub shotgun_shell_scene: Handle<Scene>,
    #[asset(path = "minigun/minigun.glb#Scene0")]
    pub minigun_scene: Handle<Scene>,
    #[asset(path = "minigun/minigun_shell.glb#Scene0")]
    pub minigun_shell_scene: Handle<Scene>,
}

#[derive(Resource)]
pub struct WeaponsResources {
    pub projectile_mesh: Handle<Mesh>,
    pub projectile_material: Handle<StandardMaterial>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    #[default]
    Pistol,
    Shotgun,
    Minigun,
}

#[derive(Component)]
struct WeaponShootAnimation {
    animate_forward: bool,
    animate_backward: bool,
    animation_speed: f32,
    progress: f32,
    initial_transform: Transform,
    target_transform: Transform,
}

#[derive(Default, Component)]
pub struct Weapon {
    weapon_type: WeaponType,
}

#[derive(Component)]
pub struct WeaponModel;

#[derive(Default, Component)]
pub struct Ammo {
    pub ammo: u32,
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
pub struct FreeFloatingWeapon {
    pub original_translation: Vec3,
}

#[derive(Bundle)]
pub struct WeaponBundle {
    pub transform_bundle: TransformBundle,
    pub inherited_visibility: InheritedVisibility,
    pub ammo: Ammo,
    pub weapon_attack_timer: WeaponAttackTimer,
    pub weapon: Weapon,
}

impl WeaponBundle {
    pub fn pistol(transform: Transform) -> Self {
        Self {
            transform_bundle: TransformBundle::from_transform(transform),
            inherited_visibility: InheritedVisibility::VISIBLE,
            ammo: Ammo { ammo: PISTOL_AMMO },
            weapon_attack_timer: WeaponAttackTimer::new(PISTOL_ATTACK_SPEED),
            weapon: Weapon {
                weapon_type: WeaponType::Pistol,
            },
        }
    }

    pub fn shotgun(transform: Transform) -> Self {
        Self {
            transform_bundle: TransformBundle::from_transform(transform),
            inherited_visibility: InheritedVisibility::VISIBLE,
            ammo: Ammo { ammo: SHOTGUN_AMMO },
            weapon_attack_timer: WeaponAttackTimer::new(SHOTGUN_ATTACK_SPEED),
            weapon: Weapon {
                weapon_type: WeaponType::Shotgun,
            },
        }
    }

    pub fn minigun(transform: Transform) -> Self {
        Self {
            transform_bundle: TransformBundle::from_transform(transform),
            inherited_visibility: InheritedVisibility::VISIBLE,
            ammo: Ammo { ammo: MINIGUN_AMMO },
            weapon_attack_timer: WeaponAttackTimer::new(MINIGUN_ATTACK_SPEED),
            weapon: Weapon {
                weapon_type: WeaponType::Minigun,
            },
        }
    }
}

impl Default for WeaponBundle {
    fn default() -> Self {
        Self {
            transform_bundle: TransformBundle::default(),
            inherited_visibility: InheritedVisibility::VISIBLE,
            ammo: Ammo::default(),
            weapon_attack_timer: WeaponAttackTimer::new(0.0),
            weapon: Weapon::default(),
        }
    }
}

#[derive(Default, Component)]
pub struct Projectile {
    pub direction: Vec3,
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub pbr_bundle: PbrBundle,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub velocity: Velocity,
    pub projectile: Projectile,
    pub damage: Damage,

    pub level_object: LevelObject,
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            pbr_bundle: PbrBundle::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_PROJECTILES,
                COLLISION_GROUP_LEVEL | COLLISION_GROUP_PLAYER | COLLISION_GROUP_ENEMY,
            ),
            active_events: ActiveEvents::COLLISION_EVENTS,
            velocity: Velocity::default(),
            projectile: Projectile::default(),
            damage: Damage::default(),

            level_object: LevelObject,
        }
    }
}

#[derive(Bundle)]
pub struct ShellBundle {
    pub scene_bundle: SceneBundle,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub velocity: Velocity,
    pub friction: Friction,

    pub level_object: LevelObject,
}

impl Default for ShellBundle {
    fn default() -> Self {
        Self {
            scene_bundle: SceneBundle::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(DEFAULT_CLIP_LENGTH, DEFAULT_CLIP_SIZE, DEFAULT_CLIP_SIZE),
            velocity: Velocity::default(),
            friction: Friction {
                coefficient: 100.0,
                ..default()
            },

            level_object: LevelObject,
        }
    }
}

#[derive(Bundle)]
pub struct FreeFloatingWeaponBundle {
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub free_floating_weapon: FreeFloatingWeapon,

    pub level_object: LevelObject,
}

impl FreeFloatingWeaponBundle {
    pub fn new(original_translation: Vec3) -> Self {
        Self {
            collider: Collider::ball(FREE_FLOATING_WEAPON_COLLIDER_RADIUS),
            collision_groups: CollisionGroups::new(COLLISION_GROUP_PICKUP, COLLISION_GROUP_PLAYER),
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            free_floating_weapon: FreeFloatingWeapon {
                original_translation,
            },

            level_object: LevelObject,
        }
    }
}

impl WeaponAttackTimer {
    pub fn new(seconds: f32) -> Self {
        Self {
            attack_timer: Timer::new(
                std::time::Duration::from_secs_f32(seconds),
                TimerMode::Repeating,
            ),
        }
    }
}

fn init_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let projectile_mesh = meshes.add(
        shape::UVSphere {
            radius: DEFAULT_PROJECTILE_SIZE,
            ..default()
        }
        .into(),
    );
    let projectile_material = materials.add(Color::GOLD.into());

    commands.insert_resource(WeaponsResources {
        projectile_mesh,
        projectile_material,
    });
}

pub fn spawn_weapon(
    weapons_assets: &WeaponAssets,
    weapon_type: WeaponType,
    commands: &mut Commands,
    transform: Transform,
) {
    match weapon_type {
        WeaponType::Pistol => {
            commands
                .spawn((
                    WeaponBundle::pistol(transform),
                    FreeFloatingWeaponBundle::new(transform.translation),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        SceneBundle {
                            scene: weapons_assets.pistol_scene.clone(),
                            ..default()
                        },
                        WeaponModel,
                    ));
                });
        }

        WeaponType::Shotgun => {
            commands
                .spawn((
                    WeaponBundle::shotgun(transform),
                    FreeFloatingWeaponBundle::new(transform.translation),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        SceneBundle {
                            scene: weapons_assets.shotgun_scene.clone(),
                            ..default()
                        },
                        WeaponModel,
                    ));
                });
        }
        WeaponType::Minigun => {
            commands
                .spawn((
                    WeaponBundle::minigun(transform),
                    FreeFloatingWeaponBundle::new(transform.translation),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        SceneBundle {
                            scene: weapons_assets.minigun_scene.clone(),
                            ..default()
                        },
                        WeaponModel,
                    ));
                });
        }
    }
}

fn update_attack_timers(time: Res<Time>, mut timers: Query<&mut WeaponAttackTimer>) {
    for mut timer in timers.iter_mut() {
        timer.attack_timer.tick(time.delta());
    }
}

fn update_free_floating_weapons(
    time: Res<Time>,
    mut weapons: Query<(&FreeFloatingWeapon, &mut Transform)>,
) {
    for (floating, mut weapon_transform) in weapons.iter_mut() {
        weapon_transform.translation = floating.original_translation
            + Vec3::NEG_Z
                * FREE_FLOATING_WEAPON_AMPLITUDE_MODIFIER
                * (time.elapsed().as_secs_f32() * FREE_FLOATING_WEAPON_BOUNCE_SPEED_MODIFIER).sin();
        weapon_transform.rotate_z(time.delta_seconds() * FREE_FLOATING_WEAPON_ROTATION_SPEED);
    }
}

fn weapon_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut animated_weapons: Query<(Entity, &mut WeaponShootAnimation, &mut Transform)>,
) {
    for (hud, mut animation, mut transform) in animated_weapons.iter_mut() {
        animation.progress += time.delta_seconds() * animation.animation_speed;

        if animation.animate_forward {
            transform.translation = animation
                .initial_transform
                .translation
                .lerp(animation.target_transform.translation, animation.progress);
            transform.rotation = animation
                .initial_transform
                .rotation
                .lerp(animation.target_transform.rotation, animation.progress);

            if 1.0 <= animation.progress {
                if animation.animate_backward {
                    animation.progress = 0.0;
                    animation.animate_forward = false;
                } else {
                    let Some(mut e) = commands.get_entity(hud) else {
                        return;
                    };
                    e.remove::<WeaponShootAnimation>();
                }
            }
        } else if animation.animate_backward {
            transform.translation = animation
                .target_transform
                .translation
                .lerp(animation.initial_transform.translation, animation.progress);
            transform.rotation = animation
                .target_transform
                .rotation
                .lerp(animation.initial_transform.rotation, animation.progress);

            if 1.0 <= animation.progress {
                let Some(mut e) = commands.get_entity(hud) else {
                    return;
                };
                e.remove::<WeaponShootAnimation>();
            }
        }
    }
}

fn weapon_shoot(
    weapons: Query<(&Weapon, &Children)>,
    weapon_models: Query<&Transform, With<WeaponModel>>,
    weapon_assets: Res<WeaponAssets>,
    weapon_resources: Res<WeaponsResources>,
    mut commands: Commands,
    mut shoot_event: EventReader<ShootEvent>,
) {
    for e in shoot_event.read() {
        if let Ok((weapon, weapon_children)) = weapons.get(e.weapon_entity) {
            let (
                projectile_offset_scale,
                projectile_velocity,
                damage,
                shell_initial_velocity,
                shell_scene,
                animation_speed,
                animate_forward,
                animate_backward,
                animation_translation,
                animation_rotation_x,
                animation_rotation_y,
            ) = match weapon.weapon_type {
                WeaponType::Pistol => (
                    PISTOL_PROJECTILE_OFFSET_SCALE,
                    PISTOL_PROJECTILE_VELOCITY,
                    PISTOL_DAMAGE,
                    PISTOL_SHELL_INITIAL_VELOCITY,
                    weapon_assets.pistol_shell_scene.clone(),
                    PISTOL_ANIMATION_SPEED,
                    PISTOL_ANIMATION_FORWARD,
                    PISTOL_ANIMATION_BACKWARD,
                    PISTOL_ANIMATION_TARGET_OFFSET,
                    PISTOL_ANIMATION_TARGET_ROTATION_X,
                    PISTOL_ANIMATION_TARGET_ROTATION_Y,
                ),
                WeaponType::Shotgun => (
                    SHOTGUN_PROJECTILE_OFFSET_SCALE,
                    SHOTGUN_PROJECTILE_VELOCITY,
                    SHOTGUN_DAMAGE,
                    SHOTGUN_SHELL_INITIAL_VELOCITY,
                    weapon_assets.shotgun_shell_scene.clone(),
                    SHOTGUN_ANIMATION_SPEED,
                    SHOTGUN_ANIMATION_FORWARD,
                    SHOTGUN_ANIMATION_BACKWARD,
                    SHOTGUN_ANIMATION_TARGET_OFFSET,
                    SHOTGUN_ANIMATION_TARGET_ROTATION_X,
                    SHOTGUN_ANIMATION_TARGET_ROTATION_Y,
                ),
                WeaponType::Minigun => (
                    MINIGUN_PROJECTILE_OFFSET_SCALE,
                    MINIGUN_PROJECTILE_VELOCITY,
                    MINIGUN_DAMAGE,
                    MINIGUN_SHELL_INITIAL_VELOCITY,
                    weapon_assets.minigun_shell_scene.clone(),
                    MINIGUN_ANIMATION_SPEED,
                    MINIGUN_ANIMATION_FORWARD,
                    MINIGUN_ANIMATION_BACKWARD,
                    MINIGUN_ANIMATION_TARGET_OFFSET,
                    MINIGUN_ANIMATION_TARGET_ROTATION_X,
                    MINIGUN_ANIMATION_TARGET_ROTATION_Y,
                ),
            };

            let right = e.direction.cross(Vec3::Z);

            // spawn projectiles
            match weapon.weapon_type {
                WeaponType::Pistol => {
                    let projectile_translation =
                        e.weapon_translation + e.direction * projectile_offset_scale;
                    commands.spawn(ProjectileBundle {
                        pbr_bundle: PbrBundle {
                            mesh: weapon_resources.projectile_mesh.clone(),
                            material: weapon_resources.projectile_material.clone(),
                            transform: Transform::from_translation(projectile_translation),
                            ..default()
                        },
                        collider: Collider::ball(DEFAULT_PROJECTILE_SIZE),
                        velocity: Velocity {
                            linvel: e.direction * projectile_velocity,
                            ..default()
                        },
                        damage: Damage { damage },
                        projectile: Projectile {
                            direction: e.direction,
                        },
                        ..default()
                    });
                }
                WeaponType::Shotgun => {
                    let projectile_translation =
                        e.weapon_translation + e.direction * projectile_offset_scale;

                    let left_barrel = projectile_translation - right / 2.0;
                    for modifier in [
                        right / 3.0 + Vec3::Z / 3.0,
                        -right / 3.0 + Vec3::Z / 3.0,
                        right / 3.0 - Vec3::Z / 3.0,
                        -right / 3.0 - Vec3::Z / 3.0,
                    ] {
                        let projectile_translation = left_barrel + modifier;
                        commands.spawn(ProjectileBundle {
                            pbr_bundle: PbrBundle {
                                mesh: weapon_resources.projectile_mesh.clone(),
                                material: weapon_resources.projectile_material.clone(),
                                transform: Transform::from_translation(projectile_translation),
                                ..default()
                            },
                            collider: Collider::ball(DEFAULT_PROJECTILE_SIZE),
                            velocity: Velocity {
                                linvel: e.direction * projectile_velocity,
                                ..default()
                            },
                            damage: Damage { damage },
                            projectile: Projectile {
                                direction: e.direction,
                            },
                            ..default()
                        });
                    }

                    let right_barrel = projectile_translation + right / 2.0;
                    for modifier in [
                        right / 3.0 + Vec3::Z / 3.0,
                        -right / 3.0 + Vec3::Z / 3.0,
                        right / 3.0 - Vec3::Z / 3.0,
                        -right / 3.0 - Vec3::Z / 3.0,
                    ] {
                        let projectile_translation = right_barrel + modifier;
                        commands.spawn(ProjectileBundle {
                            pbr_bundle: PbrBundle {
                                mesh: weapon_resources.projectile_mesh.clone(),
                                material: weapon_resources.projectile_material.clone(),
                                transform: Transform::from_translation(projectile_translation),
                                ..default()
                            },
                            collider: Collider::ball(DEFAULT_PROJECTILE_SIZE),
                            velocity: Velocity {
                                linvel: e.direction * projectile_velocity,
                                ..default()
                            },
                            damage: Damage { damage },
                            projectile: Projectile {
                                direction: e.direction,
                            },
                            ..default()
                        });
                    }
                }
                WeaponType::Minigun => {
                    let projectile_translation =
                        e.weapon_translation + e.direction * projectile_offset_scale;

                    let left_barrel = projectile_translation - right / 2.0;
                    commands.spawn(ProjectileBundle {
                        pbr_bundle: PbrBundle {
                            mesh: weapon_resources.projectile_mesh.clone(),
                            material: weapon_resources.projectile_material.clone(),
                            transform: Transform::from_translation(left_barrel),
                            ..default()
                        },
                        collider: Collider::ball(DEFAULT_PROJECTILE_SIZE),
                        velocity: Velocity {
                            linvel: e.direction * projectile_velocity,
                            ..default()
                        },
                        damage: Damage { damage },
                        projectile: Projectile {
                            direction: e.direction,
                        },
                        ..default()
                    });

                    let right_barrel = projectile_translation + right / 2.0;
                    commands.spawn(ProjectileBundle {
                        pbr_bundle: PbrBundle {
                            mesh: weapon_resources.projectile_mesh.clone(),
                            material: weapon_resources.projectile_material.clone(),
                            transform: Transform::from_translation(right_barrel),
                            ..default()
                        },
                        collider: Collider::ball(DEFAULT_PROJECTILE_SIZE),
                        velocity: Velocity {
                            linvel: e.direction * projectile_velocity,
                            ..default()
                        },
                        damage: Damage { damage },
                        projectile: Projectile {
                            direction: e.direction,
                        },
                        ..default()
                    });
                }
            }

            // spawn shell
            let shell_direction = right + Vec3::Z;
            match weapon.weapon_type {
                WeaponType::Pistol => {
                    commands.spawn(ShellBundle {
                        scene_bundle: SceneBundle {
                            scene: shell_scene,
                            transform: Transform::from_translation(e.weapon_translation)
                                .with_scale(Vec3::new(2.0, 2.0, 2.0)),
                            ..default()
                        },
                        velocity: Velocity {
                            linvel: shell_direction * shell_initial_velocity,
                            ..default()
                        },
                        ..default()
                    });
                }
                WeaponType::Shotgun | WeaponType::Minigun => {
                    commands.spawn(ShellBundle {
                        scene_bundle: SceneBundle {
                            scene: shell_scene.clone(),
                            transform: Transform::from_translation(
                                e.weapon_translation - right / 2.0,
                            )
                            .with_scale(Vec3::new(2.0, 2.0, 2.0)),
                            ..default()
                        },
                        velocity: Velocity {
                            linvel: shell_direction * shell_initial_velocity,
                            ..default()
                        },
                        ..default()
                    });
                    commands.spawn(ShellBundle {
                        scene_bundle: SceneBundle {
                            scene: shell_scene,
                            transform: Transform::from_translation(
                                e.weapon_translation - right / 2.0,
                            )
                            .with_scale(Vec3::new(2.0, 2.0, 2.0)),
                            ..default()
                        },
                        velocity: Velocity {
                            linvel: shell_direction * shell_initial_velocity,
                            ..default()
                        },
                        ..default()
                    });
                }
            }

            // start shooting animation
            let weapon_model = weapon_children[0];
            let Ok(weapon_model_transform) = weapon_models.get(weapon_model) else {
                continue;
            };
            let initial_transform = *weapon_model_transform;
            let mut target_transform = initial_transform;
            target_transform.translation += animation_translation;
            target_transform.rotation *= Quat::from_rotation_x(animation_rotation_x)
                * Quat::from_rotation_y(animation_rotation_y);
            let Some(mut e) = commands.get_entity(weapon_model) else {
                continue;
            };
            e.insert(WeaponShootAnimation {
                animate_forward,
                animate_backward,
                animation_speed,
                progress: 0.0,
                initial_transform,
                target_transform,
            });
        }
    }
}

// fn display_events(
//     // rapier_context: Res<bevy_rapier3d::plugin::RapierContext>,
//     mut collision_events: EventReader<bevy_rapier3d::pipeline::CollisionEvent>,
// ) {
//     // for p in rapier_context.contact_pairs() {
//     //     println!("pair: {:?} : {:?}", p.collider1(), p.collider2());
//     // }
//     for collision_event in collision_events.read() {
//         println!("Received collision event: {:?}", collision_event);
//     }
// }
