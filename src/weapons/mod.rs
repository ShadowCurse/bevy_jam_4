use bevy::prelude::*;

pub mod pistol;

const FREE_FLOATING_WEAPON_ROTATION_SPEED: f32 = 0.4;
const FREE_FLOATING_WEAPON_AMPLITUDE_MODIFIER: f32 = 0.5;
const FREE_FLOATING_WEAPON_BOUNCE_SPEED_MODIFIER: f32 = 2.0;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShootEvent>();
        app.add_plugins(pistol::PistolPlugin);
        app.add_systems(Startup, init_resources);
        app.add_systems(
            Update,
            (
                update_attack_timers,
                update_free_floating_weapons,
                // display_events,
            ),
        );
    }
}

#[derive(Resource)]
pub struct WeaponsResources {
    pistol_mesh: Handle<Mesh>,
    pistol_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Projectile;

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
    // forward = -Z
    let pistol_mesh = meshes.add(shape::Box::new(0.1, 0.2, 1.5).into());
    let pistol_material = materials.add(Color::GREEN.into());

    commands.insert_resource(WeaponsResources {
        pistol_mesh,
        pistol_material,
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

// fn display_events(
//     rapier_context: Res<RapierContext>,
//     mut collision_events: EventReader<CollisionEvent>,
// ) {
//     for p in rapier_context.contact_pairs() {
//         println!("pair: {:?} : {:?}", p.collider1(), p.collider2());
//     }
//     for collision_event in collision_events.read() {
//         println!("Received collision event: {:?}", collision_event);
//     }
// }
