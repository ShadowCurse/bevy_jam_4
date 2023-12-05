use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::CollisionEventFlags};
use rand::Rng;

use crate::{
    enemies::{fridge::spawn_fridge, EnemiesResources, Enemy},
    player::spawn_player,
    weapons::{pistol::spawn_pistol, Projectile, WeaponsResources},
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_LEVEL, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_PROJECTILES,
};

use self::door::{Door, DoorBundle, DoorType};

mod door;

const LEVEL_SIZE: f32 = 200.0;
const COLUMN_SIZE: f32 = 5.0;
const COLUMN_HIGHT: f32 = 10.0;
const GRID_SIZE: usize = (LEVEL_SIZE / COLUMN_SIZE) as usize;
const FILL_AMOUNT: f32 = 0.02;
const STRIP_LENGTH: u32 = 3;

const LEVEL_WEAPON_SPAWNS: u32 = 4;
const LEVEL_ENEMIES: u32 = 2;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelFinished>();
        app.add_event::<LevelSwitch>();
        app.add_plugins(door::DoorPlugin);
        app.add_systems(Startup, init);
        app.add_systems(PostStartup, spawn_initial_level);
        app.add_systems(
            Update,
            (
                level_progress,
                level_switch,
                collision_level_object_projectiles,
            ),
        );
    }
}

#[derive(Resource)]
struct LevelResources {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,
    column_mesh: Handle<Mesh>,
    column_material: Handle<StandardMaterial>,
    door_mesh: Handle<Mesh>,
    door_closed_material: Handle<StandardMaterial>,
    door_open_material: Handle<StandardMaterial>,
}

// This component needs to be attached to
// all entities of the level. It will be
// used to clean up all entities from
// old level.
#[derive(Component)]
pub struct LevelObject;

#[derive(Resource)]
struct LevelState {
    finished: bool,
    translation: Vec3,
    old_level_objects: Vec<Entity>,
}

#[derive(Event)]
struct LevelFinished;

#[derive(Event)]
struct LevelSwitch {
    exit_door: Door,
}

#[derive(Component)]
pub struct LevelCollider;

#[derive(Bundle)]
pub struct LevelColliderBundle {
    pub pbr_bundle: PbrBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_collision_types: ActiveCollisionTypes,
    pub rigid_body: RigidBody,
    pub level_collider: LevelCollider,

    pub level_object: LevelObject,
}

impl Default for LevelColliderBundle {
    fn default() -> Self {
        Self {
            pbr_bundle: PbrBundle::default(),
            collider: Collider::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_LEVEL,
                COLLISION_GROUP_ENEMY | COLLISION_GROUP_PLAYER | COLLISION_GROUP_PROJECTILES,
            ),
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            rigid_body: RigidBody::Fixed,
            level_collider: LevelCollider,

            level_object: LevelObject,
        }
    }
}

impl LevelColliderBundle {
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        collider: Collider,
    ) -> Self {
        Self {
            pbr_bundle: PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            collider,
            ..default()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellType {
    Empty,
    Door(Door),
    Column,
    Weapon,
    Enemy,
    Player,
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_mesh = meshes.add(shape::Box::new(LEVEL_SIZE, LEVEL_SIZE, 1.0).into());
    let floor_material = materials.add(Color::GRAY.into());

    let column_mesh = meshes.add(shape::Box::new(COLUMN_SIZE, COLUMN_SIZE, COLUMN_HIGHT).into());
    let column_material = materials.add(Color::DARK_GRAY.into());

    let door_mesh = meshes.add(shape::Box::new(COLUMN_SIZE, COLUMN_SIZE, COLUMN_HIGHT).into());
    let door_closed_material = materials.add(Color::RED.into());
    let door_open_material = materials.add(Color::BLUE.into());

    commands.insert_resource(LevelResources {
        floor_mesh,
        floor_material,
        column_mesh,
        column_material,
        door_mesh,
        door_closed_material,
        door_open_material,
    });

    commands.insert_resource(LevelState {
        finished: false,
        translation: Vec3::ZERO,
        old_level_objects: vec![],
    });
}

fn generate_level(previus_door: Option<Door>) -> [[CellType; GRID_SIZE]; GRID_SIZE] {
    let mut rng = rand::thread_rng();

    let mut grid = [[CellType::Empty; GRID_SIZE]; GRID_SIZE];

    // generate doors
    let mut random_exit_top = rng.gen_range(1..GRID_SIZE - 1);
    let mut random_exit_bottom = rng.gen_range(1..GRID_SIZE - 1);
    let mut random_exit_left = rng.gen_range(1..GRID_SIZE - 1);
    let mut random_exit_right = rng.gen_range(1..GRID_SIZE - 1);

    // check prevous exit and place player at mirrored door
    if let Some(door) = previus_door {
        match door.door_type {
            DoorType::Top => {
                random_exit_bottom = door.grid_pox;
                // grid[GRID_SIZE - 2][random_exit_bottom] = CellType::Player;
            }
            DoorType::Bottom => {
                random_exit_top = door.grid_pox;
                // grid[1][random_exit_top] = CellType::Player;
            }
            DoorType::Left => {
                random_exit_right = door.grid_pox;
                // grid[random_exit_right][GRID_SIZE - 2] = CellType::Player;
            }
            DoorType::Right => {
                random_exit_left = door.grid_pox;
                // grid[random_exit_left][1] = CellType::Player;
            }
        }
    } else {
        // if it is the first level place at the bottom
        grid[GRID_SIZE - 2][random_exit_bottom] = CellType::Player;
    }

    for x in 0..GRID_SIZE {
        grid[0][x] = CellType::Column;
    }
    grid[0][random_exit_top] = CellType::Door(Door {
        door_type: DoorType::Top,
        grid_pox: random_exit_top,
    });

    for x in 0..GRID_SIZE {
        grid[GRID_SIZE - 1][x] = CellType::Column;
    }
    grid[GRID_SIZE - 1][random_exit_bottom] = CellType::Door(Door {
        door_type: DoorType::Bottom,
        grid_pox: random_exit_bottom,
    });

    (0..GRID_SIZE).for_each(|y| {
        grid[y][0] = CellType::Column;
    });
    grid[random_exit_left][0] = CellType::Door(Door {
        door_type: DoorType::Left,
        grid_pox: random_exit_left,
    });

    (0..GRID_SIZE).for_each(|y| {
        grid[y][GRID_SIZE - 1] = CellType::Column;
    });
    grid[random_exit_right][GRID_SIZE - 1] = CellType::Door(Door {
        door_type: DoorType::Right,
        grid_pox: random_exit_right,
    });

    // generate walls
    let fill_cells = (GRID_SIZE as f32 * GRID_SIZE as f32 * FILL_AMOUNT) as u32;
    let num_strips = fill_cells / STRIP_LENGTH;
    for _ in 0..num_strips {
        let random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
        let random_cell_y = rng.gen_range(2..GRID_SIZE - 2);
        grid[random_cell_y][random_cell_x] = CellType::Column;

        let mut current_x: i32 = random_cell_x as i32;
        let mut current_y: i32 = random_cell_y as i32;

        for _ in 0..STRIP_LENGTH {
            let mods = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            let valid_pos = mods
                .iter()
                .filter_map(|(x_mod, y_mod)| {
                    let (next_x, next_y) = (current_x + x_mod, current_y + y_mod);
                    if next_x < 2
                        || GRID_SIZE as i32 - 2 <= next_x
                        || next_y < 2
                        || GRID_SIZE as i32 - 2 <= next_y
                    {
                        None
                    } else {
                        Some((next_x, next_y))
                    }
                })
                .collect::<Vec<_>>();

            if valid_pos.is_empty() {
                break;
            }

            let random_cell = rng.gen_range(0..valid_pos.len());
            (current_x, current_y) = valid_pos[random_cell];
            grid[current_y as usize][current_x as usize] = CellType::Column;
        }
    }

    // check if there are some "trapped" places and remove them
    for y in 2..GRID_SIZE - 2 {
        for x in 2..GRID_SIZE - 2 {
            if grid[y][x] == CellType::Empty
                && grid[y - 1][x] == CellType::Column
                && grid[y + 1][x] == CellType::Column
                && grid[y][x + 1] == CellType::Column
                && grid[y][x - 1] == CellType::Column
            {
                grid[y][x] = CellType::Column;
            }
        }
    }

    // generate weapon spawns
    for _ in 0..LEVEL_WEAPON_SPAWNS {
        let mut random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
        let mut random_cell_y = rng.gen_range(2..GRID_SIZE - 2);

        while grid[random_cell_y][random_cell_x] != CellType::Empty {
            random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
            random_cell_y = rng.gen_range(2..GRID_SIZE - 2);
        }

        grid[random_cell_y][random_cell_x] = CellType::Weapon;
    }

    // generate enemies
    for _ in 0..LEVEL_ENEMIES {
        let mut random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
        let mut random_cell_y = rng.gen_range(2..GRID_SIZE - 2);

        while grid[random_cell_y][random_cell_x] != CellType::Empty {
            random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
            random_cell_y = rng.gen_range(2..GRID_SIZE - 2);
        }

        grid[random_cell_y][random_cell_x] = CellType::Enemy;
    }

    grid
}

fn spawn_level(
    level_resources: &LevelResources,
    weapons_resources: &WeaponsResources,
    enemies_resources: &EnemiesResources,
    commands: &mut Commands,
    level_translation: Vec3,
    previus_door: Option<Door>,
) -> Vec3 {
    let grid = generate_level(previus_door);

    let level_translation = match previus_door {
        Some(door) => match door.door_type {
            DoorType::Top => level_translation + Vec3::new(0.0, -LEVEL_SIZE, 0.0),
            DoorType::Bottom => level_translation + Vec3::new(0.0, LEVEL_SIZE, 0.0),
            DoorType::Left => level_translation + Vec3::new(-LEVEL_SIZE, 0.0, 0.0),
            DoorType::Right => level_translation + Vec3::new(LEVEL_SIZE, 0.0, 0.0),
        },
        None => level_translation,
    };

    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let x_pos = (-LEVEL_SIZE / 2.0) + COLUMN_SIZE * x as f32 + COLUMN_SIZE / 2.0;
            let y_pos = (-LEVEL_SIZE / 2.0) + COLUMN_SIZE * y as f32 + COLUMN_SIZE / 2.0;
            let z_pos = COLUMN_HIGHT / 2.0;
            let translation = Vec3::new(x_pos, y_pos, z_pos);
            let transform = Transform::from_translation(translation + level_translation);

            match cell {
                CellType::Column => {
                    commands.spawn((LevelColliderBundle::new(
                        level_resources.column_mesh.clone(),
                        level_resources.column_material.clone(),
                        transform,
                        Collider::cuboid(COLUMN_SIZE / 2.0, COLUMN_SIZE / 2.0, COLUMN_HIGHT / 2.0),
                    ),));
                }
                CellType::Door(door) => {
                    commands.spawn((DoorBundle::new(
                        level_resources.door_mesh.clone(),
                        level_resources.door_closed_material.clone(),
                        transform,
                        Collider::cuboid(COLUMN_SIZE / 2.0, COLUMN_SIZE / 2.0, COLUMN_HIGHT / 2.0),
                        *door,
                    ),));
                }
                CellType::Weapon => {
                    spawn_pistol(weapons_resources, commands, transform);
                }
                CellType::Enemy => {
                    spawn_fridge(enemies_resources, weapons_resources, commands, transform);
                }
                CellType::Player => {
                    spawn_player(commands, transform);
                }
                CellType::Empty => {}
            }
        }
    }

    // floor
    commands.spawn(LevelColliderBundle::new(
        level_resources.floor_mesh.clone(),
        level_resources.floor_material.clone(),
        Transform::from_translation(level_translation),
        Collider::cuboid(LEVEL_SIZE / 2.0, LEVEL_SIZE / 2.0, 0.5),
    ));

    level_translation
}

fn spawn_initial_level(
    level_state: Res<LevelState>,
    level_resources: Res<LevelResources>,
    weapons_resources: Res<WeaponsResources>,
    enemies_resources: Res<EnemiesResources>,
    mut commands: Commands,
) {
    spawn_level(
        level_resources.as_ref(),
        weapons_resources.as_ref(),
        enemies_resources.as_ref(),
        &mut commands,
        level_state.translation,
        None,
    );
}

fn level_progress(
    enemies: Query<Entity, With<Enemy>>,
    mut level_state: ResMut<LevelState>,
    mut level_finished_events: EventWriter<LevelFinished>,
) {
    let remaining_enemies = enemies.iter().count();
    if remaining_enemies == 0 && !level_state.finished {
        level_state.finished = true;
        level_finished_events.send(LevelFinished);
    }
}

fn level_switch(
    level_resources: Res<LevelResources>,
    weapons_resources: Res<WeaponsResources>,
    enemies_resources: Res<EnemiesResources>,
    level_objects: Query<Entity, With<LevelObject>>,
    mut level_state: ResMut<LevelState>,
    mut commands: Commands,
    mut level_switch_events: EventReader<LevelSwitch>,
) {
    for event in level_switch_events.read() {
        let old_level_objects = level_objects.iter().collect::<Vec<_>>();

        let new_translation = spawn_level(
            level_resources.as_ref(),
            weapons_resources.as_ref(),
            enemies_resources.as_ref(),
            &mut commands,
            level_state.translation,
            Some(event.exit_door),
        );

        level_state.translation = new_translation;
        level_state.old_level_objects = old_level_objects;
    }
}

fn collision_level_object_projectiles(
    projectiles: Query<Entity, With<Projectile>>,
    level_objects: Query<Entity, With<LevelCollider>>,
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        let (collider_1, collider_2, flags) = match collision_event {
            CollisionEvent::Started(c1, c2, f) => (c1, c2, f),
            CollisionEvent::Stopped(c1, c2, f) => (c1, c2, f),
        };
        if flags.contains(CollisionEventFlags::REMOVED) {
            return;
        }

        let projectile = if let Ok(p) = projectiles.get(*collider_1) {
            if level_objects.get(*collider_2).is_ok() {
                p
            } else {
                continue;
            }
        } else if let Ok(p) = projectiles.get(*collider_2) {
            if level_objects.get(*collider_1).is_ok() {
                p
            } else {
                continue;
            }
        } else {
            continue;
        };
        commands.get_entity(projectile).unwrap().despawn();
    }
}
