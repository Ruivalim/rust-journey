use std::time::Duration;

use bevy::{
    color::palettes::css::{ORANGE, RED},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_egui::EguiPlugin;
use cell::Action;
use common::CellSelected;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

mod cell;
mod common;
mod food;
mod helpers;
mod ui;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, EguiPlugin))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(common::GAME_CONFIG)
        .insert_resource(common::RandomSource(ChaCha12Rng::from_entropy()))
        .insert_resource(common::CellSelected(None))
        .insert_resource(ClearColor(Color::hsl(186.0, 0.36, 0.71)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                helpers::camera::movement,
                start.run_if(input_just_pressed(KeyCode::Enter)),
                pause.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                day_cycle,
                brain_process,
                metabolism_process,
                cell_actions,
                game_tick,
                move_cells,
            ),
        )
        .add_plugins(ui::ui_plugin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_config: Res<common::GameConfig>,
) {
    let seeded_rng = ChaCha12Rng::from_entropy();

    commands.spawn(Camera2d);
    commands.insert_resource(common::DayNightCycleTimer(Timer::from_seconds(
        game_config.day_speed,
        TimerMode::Repeating,
    )));

    let food_mesh: Handle<Mesh> = meshes.add(Rectangle::new(10.0, 10.0));
    let cell_mesh: Handle<Mesh> = meshes.add(Rectangle::new(10.0, 10.0));

    commands.insert_resource(common::RandomSource(seeded_rng));
    commands.insert_resource(common::FoodMesh(food_mesh));
    commands.insert_resource(common::CellMesh(cell_mesh));
}

fn start(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_config: ResMut<common::GameConfig>,
    cell_mesh: Res<common::CellMesh>,
    food_mesh: Res<common::FoodMesh>,
    mut seeded_rng: ResMut<common::RandomSource>,
    cell_query: Query<Entity, (With<cell::Cell>, Without<food::Food>)>,
    food_query: Query<Entity, (With<food::Food>, Without<cell::Cell>)>,
) {
    game_config.paused = false;
    for food in food_query.iter() {
        commands.entity(food).despawn();
    }
    for cell in cell_query.iter() {
        commands.entity(cell).despawn();
    }

    for _ in 0..15 {
        commands
            .spawn(cell::Cell::new(
                &mut seeded_rng.0,
                &game_config,
                &cell_mesh.0,
                &mut materials,
            ))
            .observe(select_cell);
    }

    for _ in 0..100 {
        commands.spawn(food::Food::new(
            &mut seeded_rng.0,
            &game_config,
            &food_mesh.0,
            &mut materials,
        ));
    }
}

fn pause(mut game_config: ResMut<common::GameConfig>) {
    game_config.paused = !game_config.paused;
}

fn select_cell(
    trigger: Trigger<Pointer<Up>>,
    query: Query<&cell::Cell>,
    mut selected_cel: ResMut<CellSelected>,
) {
    let cell = query.get(trigger.entity()).unwrap();
    selected_cel.0 = Some(cell.clone());
}

fn game_tick(
    cell_query: Query<&cell::Cell, (With<cell::Cell>, Without<food::Food>)>,
    food_query: Query<&food::Food, (With<food::Food>, Without<cell::Cell>)>,
    selected_cel: Res<CellSelected>,
    mut gizmos: Gizmos,
    game_config: ResMut<common::GameConfig>,
) {
    let cells_snapshot: Vec<cell::Cell> = cell_query.iter().map(|cell| cell.clone()).collect();
    let foods_snapshot: Vec<food::Food> = food_query.iter().map(|food| food.clone()).collect();

    gizmos.rect_2d(
        Isometry2d::IDENTITY,
        Vec2::new(game_config.map_width + 10.0, game_config.map_height + 10.0),
        Color::hsl(208.0, 0.91, 0.09),
    );

    if let Some(selected) = &selected_cel.0 {
        for cell in cell_query.iter() {
            if cell.id.eq(&selected.id) {
                cell.draw_vision(&mut gizmos, &cells_snapshot, &foods_snapshot);
            }
        }
    }
}

fn day_cycle(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<common::DayNightCycleTimer>,
    mut cell_query: Query<&mut cell::Cell>,
    mut game_config: ResMut<common::GameConfig>,
    mut seeded_rng: ResMut<common::RandomSource>,
    food_mesh: Res<common::FoodMesh>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game_config.paused {
        return;
    }
    timer
        .0
        .set_duration(Duration::from_secs_f32(game_config.day_speed));
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        game_config.current_day += 1;
        for _ in 0..game_config.foods_per_day {
            commands.spawn(food::Food::new(
                &mut seeded_rng.0,
                &game_config,
                &food_mesh.0,
                &mut materials,
            ));
        }
        for mut cell in cell_query.iter_mut() {
            cell.age += 1;
        }
    }
}

fn brain_process(
    mut cell_query: Query<&mut cell::Cell, (With<cell::Cell>, Without<food::Food>)>,
    food_query: Query<&food::Food, (With<food::Food>, Without<cell::Cell>)>,
    mut seeded_rng: ResMut<common::RandomSource>,
    game_config: ResMut<common::GameConfig>,
) {
    if game_config.paused {
        return;
    }
    let cells_snapshot: Vec<cell::Cell> = cell_query.iter().map(|cell| cell.clone()).collect();
    let foods_snapshot: Vec<food::Food> = food_query.iter().map(|food| food.clone()).collect();

    for mut cell in cell_query.iter_mut() {
        cell.process_brain(&mut seeded_rng.0, &cells_snapshot, &foods_snapshot);
    }
}

fn metabolism_process(
    mut commands: Commands,
    mut cell_query: Query<(&mut cell::Cell, Entity)>,
    time: Res<Time>,
    mut game_config: ResMut<common::GameConfig>,
) {
    if game_config.paused {
        return;
    }
    for (mut cell, entity) in cell_query.iter_mut() {
        cell.process_metabolism(time.delta_secs());
        if cell.health <= 0.0 {
            game_config.dead_cells += 1;
            commands.entity(entity).despawn();
        }
    }
}

fn move_cells(
    mut cell_query: Query<(&mut cell::Cell, &mut Transform)>,
    time: Res<Time>,
    game_config: Res<common::GameConfig>,
) {
    if game_config.paused {
        return;
    }
    for (mut cell, mut transform) in cell_query.iter_mut() {
        cell.movement(&mut transform, &game_config, time.delta_secs());
    }
}

fn cell_actions(
    mut commands: Commands,
    mut cell_query: Query<&mut cell::Cell, (With<cell::Cell>, Without<food::Food>)>,
    food_query: Query<(&food::Food, Entity), (With<food::Food>, Without<cell::Cell>)>,
    mut seeded_rng: ResMut<common::RandomSource>,
    game_config: Res<common::GameConfig>,
    cell_mesh: Res<common::CellMesh>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game_config.paused {
        return;
    }

    for mut cell in cell_query.iter_mut() {
        if cell.energy > 90.0 {
            let offspring = cell.create_offspring(
                &mut seeded_rng.0,
                &game_config,
                &cell_mesh.0,
                &mut materials,
            );
            commands.spawn(offspring).observe(select_cell);
        }
        let cell_position = Vec2::new(cell.pos_x, cell.pos_y);
        match cell.action {
            // Action::Duplicate => {
            //     if cell.energy < 90.0 {
            //         return;
            //     }
            //     let offspring = cell.create_offspring(
            //         &mut seeded_rng.0,
            //         &game_config,
            //         &cell_mesh.0,
            //         &mut materials,
            //     );
            //     commands.spawn(offspring).observe(select_cell);
            //     // let mut nearest = 100000.0;
            //     // let mut found_mate = false;

            //     // for other_cell in cells_snapshot.iter() {
            //     //     if other_cell.id != cell.id
            //     //         && other_cell.mature
            //     //         && other_cell.action == Action::FindMate
            //     //     {
            //     //         let other_cell_position = Vec2::new(other_cell.pos_x, other_cell.pos_y);

            //     //         if cell.is_within_vision_cone(other_cell_position) {
            //     //             found_mate = true;
            //     //             let distance = other_cell_position.distance(cell_position);

            //     //             if distance < nearest {
            //     //                 cell.target_location = Some(other_cell_position);
            //     //                 nearest = distance;
            //     //             }

            //     //             if distance < 1.0 {
            //     //                 let offspring = cell.create_offspring(
            //     //                     &mut seeded_rng.0,
            //     //                     &game_config,
            //     //                     &cell_mesh.0,
            //     //                     &mut materials,
            //     //                     &other_cell,
            //     //                 );
            //     //                 commands.spawn(offspring).observe(select_cell);
            //     //                 break;
            //     //             }
            //     //         }
            //     //     }
            //     // }

            //     // if !found_mate {
            //     //     cell.random_target(&mut seeded_rng.0, &game_config);
            //     // }
            // }
            Action::GoingForFood => {
                let mut nearest = 100000.0;
                let mut found_food = false;

                for (food, food_entity) in food_query.iter() {
                    let food_position = Vec2::new(food.pos_x, food.pos_y);
                    if cell.is_within_vision_cone(food_position) {
                        let distance = food_position.distance(cell_position);
                        if distance < nearest {
                            found_food = true;
                            cell.target_location = Some(food_position);
                            nearest = distance;
                        }
                        if distance < 10.0 {
                            cell.eat(&food);
                            commands.entity(food_entity).despawn();
                            cell.target_location = None;
                        }
                    }
                }

                if !found_food {
                    if cell.target_location.is_none()
                        || cell_position.distance(cell.target_location.unwrap()) < 10.0
                    {
                        cell.random_target(&mut seeded_rng.0, &game_config);
                    }
                }
            }
            Action::MovingAround => {
                if cell.target_location.is_none()
                    || cell_position.distance(cell.target_location.unwrap()) < 10.0
                {
                    cell.random_target(&mut seeded_rng.0, &game_config);
                }
            }
        }
    }
}
