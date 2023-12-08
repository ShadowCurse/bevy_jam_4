use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    damage::Damage, level::LevelObject, GlobalState, COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL,
    COLLISION_GROUP_PICKUP, COLLISION_GROUP_PLAYER, COLLISION_GROUP_PROJECTILES,
};

pub mod pistol;

const DEFAULT_PROJECTILE_SIZE: f32 = 0.2;
const DEFAULT_CLIP_SIZE: f32 = 0.01;
const DEFAULT_CLIP_LENGTH: f32 = 0.02;

const FREE_FLOATING_WEAPON_COLLIDER_RADIUS: f32 = 0.8;
const FREE_FLOATING_WEAPON_ROTATION_SPEED: f32 = 0.4;
const FREE_FLOATING_WEAPON_AMPLITUDE_MODIFIER: f32 = 0.5;
const FREE_FLOATING_WEAPON_BOUNCE_SPEED_MODIFIER: f32 = 2.0;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, WeaponAssets>(GlobalState::AssetLoading);

        app.add_event::<ShootEvent>();

        app.add_plugins(pistol::PistolPlugin);

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

#[derive(Component)]
struct WeaponShootAnimation {
    animate_forward: bool,
    animation_speed: f32,
    progress: f32,
    initial_transform: Transform,
    target_transform: Transform,
}

#[derive(Component)]
pub struct Ammo {
    pub ammo: u32,
}

#[derive(Component)]
pub struct Projectile;

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
            collider: Collider::ball(DEFAULT_PROJECTILE_SIZE),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_PROJECTILES,
                COLLISION_GROUP_LEVEL | COLLISION_GROUP_PLAYER | COLLISION_GROUP_ENEMY,
            ),
            active_events: ActiveEvents::COLLISION_EVENTS,
            velocity: Velocity::default(),
            projectile: Projectile,
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

        let (it, tt) = if animation.animate_forward {
            (&animation.initial_transform, &animation.target_transform)
        } else {
            (&animation.target_transform, &animation.initial_transform)
        };
        transform.translation = it.translation.lerp(tt.translation, animation.progress);
        transform.rotation = it.rotation.lerp(tt.rotation, animation.progress);

        if 1.0 <= animation.progress {
            if animation.animate_forward {
                animation.progress = 0.0;
                animation.animate_forward = false;
            } else {
                let Some(mut e) = commands.get_entity(hud) else {
                    return;
                };
                e.remove::<WeaponShootAnimation>();
            }
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
