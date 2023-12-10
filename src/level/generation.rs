use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::{
    enemies::{spawn_enemy, EnemyAssets, EnemyType},
    player::{spawn_player, PlayerResources},
    ui::UiResources,
    weapons::{spawn_weapon, WeaponAssets, WeaponType},
};

use super::{
    door::{spawn_door, Door, DoorState, DoorType},
    spawn_light, LevelAssets, LevelColliderBundle, LevelObject, LevelResources, LevelType,
    COLUMN_HIGHT, COLUMN_SIZE, FILL_AMOUNT, FLOOR_THICKNESS, GRID_SIZE, LEVEL_ENEMIES,
    LEVEL_LIGHTS_COVERAGE, LEVEL_SIZE, LEVEL_SMALL_ENEMIES_PERCENT,
    LEVEL_WEAPON_PISTOL_SPAWN_THRESHOLD, LEVEL_WEAPON_SHOTGUN_SPAWN_THRESHOLD, LEVEL_WEAPON_SPAWNS,
    STRIP_LENGTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellType {
    Empty,
    Door(Door),
    Column,
    Light,
    Weapon(WeaponType),
    Enemy(EnemyType),
    Player,
}

// ^ y
// |
// -->x
fn generate_boss_level(previus_door: Option<Door>) -> [[CellType; GRID_SIZE]; GRID_SIZE] {
    let mut rng = rand::thread_rng();

    // row order
    let mut grid = [[CellType::Empty; GRID_SIZE]; GRID_SIZE];

    // generate border
    for x in 0..GRID_SIZE {
        grid[0][x] = CellType::Column;
    }
    for x in 0..GRID_SIZE {
        grid[GRID_SIZE - 1][x] = CellType::Column;
    }
    (0..GRID_SIZE).for_each(|y| {
        grid[y][0] = CellType::Column;
    });
    (0..GRID_SIZE).for_each(|y| {
        grid[y][GRID_SIZE - 1] = CellType::Column;
    });

    let door = previus_door.unwrap();
    match door.door_type {
        DoorType::Top => {
            grid[GRID_SIZE - 1][door.grid_pos] = CellType::Door(Door {
                door_type: DoorType::Bottom,
                door_state: DoorState::TemporaryOpen,
                grid_pos: door.grid_pos,
            });
        }
        DoorType::Bottom => {
            grid[0][door.grid_pos] = CellType::Door(Door {
                door_type: DoorType::Top,
                door_state: DoorState::TemporaryOpen,
                grid_pos: door.grid_pos,
            });
        }
        DoorType::Left => {
            grid[door.grid_pos][GRID_SIZE - 1] = CellType::Door(Door {
                door_type: DoorType::Right,
                door_state: DoorState::TemporaryOpen,
                grid_pos: door.grid_pos,
            });
        }
        DoorType::Right => {
            grid[door.grid_pos][0] = CellType::Door(Door {
                door_type: DoorType::Left,
                door_state: DoorState::TemporaryOpen,
                grid_pos: door.grid_pos,
            });
        }
    }

    let middle = GRID_SIZE / 2;
    // Big
    grid[middle][middle] = CellType::Enemy(EnemyType::Big);

    // Mid
    for x in middle - 1..=middle + 1 {
        grid[middle + 1][x] = CellType::Enemy(EnemyType::Mid);
    }
    for x in middle - 1..=middle + 1 {
        grid[middle - 1][x] = CellType::Enemy(EnemyType::Mid);
    }
    for y in middle - 1..=middle + 1 {
        grid[y][middle + 1] = CellType::Enemy(EnemyType::Mid);
    }
    for y in middle - 1..=middle + 1 {
        grid[y][middle - 1] = CellType::Enemy(EnemyType::Mid);
    }

    // Small
    for x in middle - 2..=middle + 2 {
        grid[middle + 2][x] = CellType::Enemy(EnemyType::Small);
    }
    for x in middle - 2..=middle + 2 {
        grid[middle - 2][x] = CellType::Enemy(EnemyType::Small);
    }
    for y in middle - 2..=middle + 2 {
        grid[y][middle + 2] = CellType::Enemy(EnemyType::Small);
    }
    for y in middle - 2..=middle + 2 {
        grid[y][middle - 2] = CellType::Enemy(EnemyType::Small);
    }

    // Top right corner
    grid[4][GRID_SIZE - 6] = CellType::Column;
    grid[5][GRID_SIZE - 5] = CellType::Column;
    grid[6][GRID_SIZE - 4] = CellType::Column;

    grid[4][GRID_SIZE - 5] = CellType::Weapon(WeaponType::Shotgun);
    grid[5][GRID_SIZE - 4] = CellType::Weapon(WeaponType::Shotgun);

    // Botton left corner
    grid[GRID_SIZE - 4][6] = CellType::Column;
    grid[GRID_SIZE - 5][5] = CellType::Column;
    grid[GRID_SIZE - 6][4] = CellType::Column;

    grid[GRID_SIZE - 4][5] = CellType::Weapon(WeaponType::Shotgun);
    grid[GRID_SIZE - 5][4] = CellType::Weapon(WeaponType::Shotgun);

    // Top left corner
    grid[4][4] = CellType::Column;

    grid[4][5] = CellType::Column;
    grid[4][6] = CellType::Column;
    grid[4][7] = CellType::Column;

    grid[5][4] = CellType::Column;
    grid[6][4] = CellType::Column;
    grid[7][4] = CellType::Column;

    grid[3][5] = CellType::Weapon(WeaponType::Minigun);
    grid[5][3] = CellType::Weapon(WeaponType::Minigun);

    // Bottom right corner
    grid[GRID_SIZE - 4][GRID_SIZE - 4] = CellType::Column;

    grid[GRID_SIZE - 4][GRID_SIZE - 5] = CellType::Column;
    grid[GRID_SIZE - 4][GRID_SIZE - 6] = CellType::Column;
    grid[GRID_SIZE - 4][GRID_SIZE - 7] = CellType::Column;

    grid[GRID_SIZE - 5][GRID_SIZE - 4] = CellType::Column;
    grid[GRID_SIZE - 6][GRID_SIZE - 4] = CellType::Column;
    grid[GRID_SIZE - 7][GRID_SIZE - 4] = CellType::Column;

    grid[GRID_SIZE - 3][GRID_SIZE - 5] = CellType::Weapon(WeaponType::Minigun);
    grid[GRID_SIZE - 5][GRID_SIZE - 3] = CellType::Weapon(WeaponType::Minigun);

    grid
}

// ^ y
// |
// -->x
fn generate_normal_level(previus_door: Option<Door>) -> [[CellType; GRID_SIZE]; GRID_SIZE] {
    let mut rng = rand::thread_rng();

    // row order
    let mut grid = [[CellType::Empty; GRID_SIZE]; GRID_SIZE];

    // generate border
    for x in 0..GRID_SIZE {
        grid[0][x] = CellType::Column;
    }
    for x in 0..GRID_SIZE {
        grid[GRID_SIZE - 1][x] = CellType::Column;
    }
    (0..GRID_SIZE).for_each(|y| {
        grid[y][0] = CellType::Column;
    });
    (0..GRID_SIZE).for_each(|y| {
        grid[y][GRID_SIZE - 1] = CellType::Column;
    });

    // generate doors
    let mut door_top_pos = rng.gen_range(2..GRID_SIZE - 2);
    let mut door_top_state = DoorState::Locked;

    let mut door_bottom_pos = rng.gen_range(2..GRID_SIZE - 2);
    let mut door_bottom_state = DoorState::Locked;

    let mut door_left_pos = rng.gen_range(2..GRID_SIZE - 2);
    let mut door_left_state = DoorState::Locked;

    let mut door_right_pos = rng.gen_range(2..GRID_SIZE - 2);
    let mut door_right_state = DoorState::Locked;

    // check prevous exit and place player at mirrored door
    if let Some(door) = previus_door {
        match door.door_type {
            DoorType::Top => {
                door_bottom_pos = door.grid_pos;
                door_bottom_state = DoorState::TemporaryOpen;
            }
            DoorType::Bottom => {
                door_top_pos = door.grid_pos;
                door_top_state = DoorState::TemporaryOpen;
            }
            DoorType::Left => {
                door_right_pos = door.grid_pos;
                door_right_state = DoorState::TemporaryOpen;
            }
            DoorType::Right => {
                door_left_pos = door.grid_pos;
                door_left_state = DoorState::TemporaryOpen;
            }
        }
    } else {
        // if it is the first level place at the bottom
        door_top_state = DoorState::Unlocked;
        grid[1][door_top_pos] = CellType::Player;
    }
    grid[0][door_top_pos] = CellType::Door(Door {
        door_type: DoorType::Top,
        door_state: door_top_state,
        grid_pos: door_top_pos,
    });

    grid[GRID_SIZE - 1][door_bottom_pos] = CellType::Door(Door {
        door_type: DoorType::Bottom,
        door_state: door_bottom_state,
        grid_pos: door_bottom_pos,
    });

    grid[door_left_pos][0] = CellType::Door(Door {
        door_type: DoorType::Left,
        door_state: door_left_state,
        grid_pos: door_left_pos,
    });

    grid[door_right_pos][GRID_SIZE - 1] = CellType::Door(Door {
        door_type: DoorType::Right,
        door_state: door_right_state,
        grid_pos: door_right_pos,
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

        let random = rng.gen_range(0.0..1.0);
        if random < LEVEL_WEAPON_PISTOL_SPAWN_THRESHOLD {
            grid[random_cell_y][random_cell_x] = CellType::Weapon(WeaponType::Pistol);
        }
        if random < LEVEL_WEAPON_SHOTGUN_SPAWN_THRESHOLD {
            grid[random_cell_y][random_cell_x] = CellType::Weapon(WeaponType::Shotgun);
        } else {
            grid[random_cell_y][random_cell_x] = CellType::Weapon(WeaponType::Minigun);
        }
    }

    // generate enemies
    for _ in 0..LEVEL_ENEMIES {
        let mut random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
        let mut random_cell_y = rng.gen_range(2..GRID_SIZE - 2);

        while grid[random_cell_y][random_cell_x] != CellType::Empty {
            random_cell_x = rng.gen_range(2..GRID_SIZE - 2);
            random_cell_y = rng.gen_range(2..GRID_SIZE - 2);
        }

        if rng.gen_bool(LEVEL_SMALL_ENEMIES_PERCENT) {
            grid[random_cell_y][random_cell_x] = CellType::Enemy(EnemyType::Small);
        } else {
            grid[random_cell_y][random_cell_x] = CellType::Enemy(EnemyType::Mid);
        }
    }

    // generate lights
    for y in (2..GRID_SIZE - 2).step_by(3) {
        for x in (2..GRID_SIZE - 2).step_by(3) {
            if grid[y][x] == CellType::Empty && rng.gen_bool(LEVEL_LIGHTS_COVERAGE) {
                grid[y][x] = CellType::Light;
            }
        }
    }

    // for row in grid.iter() {
    //     for cell in row.iter() {
    //         match cell {
    //             CellType::Player => print!("p"),
    //             CellType::Enemy => print!("e"),
    //             CellType::Empty => print!(" "),
    //             CellType::Column => print!("#"),
    //             CellType::Weapon => print!("w"),
    //             CellType::Door(d) => match d.door_type {
    //                 DoorType::Bottom => print!("B"),
    //                 DoorType::Top => print!("T"),
    //                 DoorType::Left => print!("L"),
    //                 DoorType::Right => print!("R"),
    //             },
    //         }
    //     }
    //     println!();
    // }

    grid
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_level(
    ui_resources: &UiResources,
    level_assets: &LevelAssets,
    enemy_assets: &EnemyAssets,
    weapon_assets: &WeaponAssets,
    level_resources: &LevelResources,
    player_resources: &PlayerResources,
    commands: &mut Commands,
    level_translation: Vec3,
    previus_door: Option<Door>,
    level_type: LevelType,
    tutorial_level: bool,
    boss_level: bool,
) -> Vec3 {
    let mut grid = if boss_level {
        generate_boss_level(previus_door)
    } else {
        generate_normal_level(previus_door)
    };

    if tutorial_level {
        let mut player_pos = (0, 0);

        // remove all content from the level
        for y in 1..GRID_SIZE - 1 {
            for x in 1..GRID_SIZE - 1 {
                if grid[y][x] != CellType::Player {
                    grid[y][x] = CellType::Empty;
                } else {
                    player_pos = (y, x);
                }
            }
        }

        // move player back
        let new_player_pos = (player_pos.0 + 3, player_pos.1);
        grid[player_pos.0][player_pos.1] = CellType::Light;
        grid[player_pos.0 + 1][player_pos.1] = CellType::Weapon(WeaponType::Pistol);
        grid[new_player_pos.0][new_player_pos.1] = CellType::Player;

        // place walls around player
        for y in 0..GRID_SIZE {
            grid[y][new_player_pos.1 - 2] = CellType::Column;
        }
        for y in 0..GRID_SIZE {
            grid[y][new_player_pos.1 + 2] = CellType::Column;
        }
        for x in 0..GRID_SIZE {
            grid[new_player_pos.0 + 2][x] = CellType::Column;
        }
    }

    let level_translation = match previus_door {
        Some(door) => match door.door_type {
            DoorType::Top => level_translation + Vec3::new(0.0, LEVEL_SIZE, 0.0),
            DoorType::Bottom => level_translation + Vec3::new(0.0, -LEVEL_SIZE, 0.0),
            DoorType::Left => level_translation + Vec3::new(-LEVEL_SIZE, 0.0, 0.0),
            DoorType::Right => level_translation + Vec3::new(LEVEL_SIZE, 0.0, 0.0),
        },
        None => level_translation,
    };

    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let x_pos = (-LEVEL_SIZE / 2.0) + COLUMN_SIZE * x as f32 + COLUMN_SIZE / 2.0;
            let y_pos = (LEVEL_SIZE / 2.0) - COLUMN_SIZE * y as f32 - COLUMN_SIZE / 2.0;
            let z_pos = COLUMN_HIGHT / 2.0;
            let translation = Vec3::new(x_pos, y_pos, z_pos);
            let transform = Transform::from_translation(translation + level_translation);

            match cell {
                CellType::Door(door) => {
                    spawn_door(level_resources, commands, transform, *door);
                }
                CellType::Column => {
                    commands.spawn(LevelColliderBundle::new(
                        level_resources.column_mesh.clone(),
                        level_resources.column_material.clone(),
                        transform,
                        Collider::cuboid(COLUMN_SIZE / 2.0, COLUMN_SIZE / 2.0, COLUMN_HIGHT / 2.0),
                    ));
                }
                CellType::Light => {
                    if level_type == LevelType::Covered {
                        let mut light_transform = transform;
                        light_transform.translation.z = COLUMN_HIGHT;
                        spawn_light(level_resources, commands, light_transform);
                    }
                }
                CellType::Weapon(weapon_type) => {
                    spawn_weapon(weapon_assets, *weapon_type, commands, transform);
                }
                CellType::Enemy(enemy_type) => {
                    spawn_enemy(
                        enemy_assets,
                        weapon_assets,
                        *enemy_type,
                        commands,
                        transform,
                    );
                }
                CellType::Player => {
                    // we spanw player only once, so we can give him
                    // some default skybox
                    spawn_player(
                        ui_resources,
                        player_resources,
                        level_assets.normal_skybox.clone(),
                        commands,
                        transform,
                    );
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

    // roof
    if level_type == LevelType::Covered {
        let mut roof_translation = level_translation;
        roof_translation.z += COLUMN_HIGHT + FLOOR_THICKNESS / 2.0;
        commands.spawn(LevelColliderBundle::new(
            level_resources.floor_mesh.clone(),
            level_resources.floor_material.clone(),
            Transform::from_translation(roof_translation),
            Collider::cuboid(LEVEL_SIZE / 2.0, LEVEL_SIZE / 2.0, 0.5),
        ));
    }

    level_translation
}

pub fn spawn_level_sun(level_type: LevelType, commands: &mut Commands) {
    match level_type {
        LevelType::Covered => {}
        LevelType::Open(level_color) => {
            // directional 'sun' light
            commands.spawn((
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        shadows_enabled: true,
                        color: level_color.into(),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 2.0, 0.0),
                        rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4)
                            * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                        ..default()
                    },
                    ..default()
                },
                LevelObject,
            ));
        }
    }
}
