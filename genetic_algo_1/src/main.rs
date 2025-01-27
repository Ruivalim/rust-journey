use std::time::Duration;

use bevy::prelude::*;
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
        .add_systems(Startup, setup)
        .add_systems(Update, (helpers::camera::movement, game_tick, move_cells))
        .add_systems(
            FixedUpdate,
            (
                day_night_cycle,
                brain_process,
                metabolism_process,
                cell_actions,
            ),
        )
        .add_plugins(ui::ui_plugin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_config: Res<common::GameConfig>,
) {
    let mut seeded_rng = ChaCha12Rng::from_entropy();

    commands.spawn(Camera2d);
    commands.insert_resource(common::DayNightCycleTimer(Timer::from_seconds(
        game_config.day_speed,
        TimerMode::Repeating,
    )));

    for _ in 0..15 {
        commands
            .spawn(cell::Cell::new(
                &mut seeded_rng,
                &game_config,
                &mut meshes,
                &mut materials,
            ))
            .observe(select_cell);
    }

    for _ in 0..100 {
        commands.spawn(food::Food::new(
            &mut seeded_rng,
            &game_config,
            &mut meshes,
            &mut materials,
        ));
    }

    commands.insert_resource(common::RandomSource(seeded_rng));
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
) {
    let cells_snapshot: Vec<cell::Cell> = cell_query.iter().map(|cell| cell.clone()).collect();
    let foods_snapshot: Vec<food::Food> = food_query.iter().map(|food| food.clone()).collect();

    if let Some(selected) = &selected_cel.0 {
        for cell in cell_query.iter() {
            if cell.id.eq(&selected.id) {
                cell.draw_vision(&mut gizmos, &cells_snapshot, &foods_snapshot);
            }
        }
    }
}

fn brain_process(
    mut cell_query: Query<&mut cell::Cell, (With<cell::Cell>, Without<food::Food>)>,
    food_query: Query<&food::Food, (With<food::Food>, Without<cell::Cell>)>,
    mut seeded_rng: ResMut<common::RandomSource>,
    game_config: Res<common::GameConfig>,
    mut gizmos: Gizmos,
) {
    let cells_snapshot: Vec<cell::Cell> = cell_query.iter().map(|cell| cell.clone()).collect();
    let foods_snapshot: Vec<food::Food> = food_query.iter().map(|food| food.clone()).collect();

    for mut cell in cell_query.iter_mut() {
        cell.process_brain(
            &mut seeded_rng.0,
            &cells_snapshot,
            &foods_snapshot,
            &game_config,
            &mut gizmos,
        );
    }
}

fn day_night_cycle(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<common::DayNightCycleTimer>,
    mut cell_query: Query<&mut cell::Cell>,
    mut game_config: ResMut<common::GameConfig>,
    mut seeded_rng: ResMut<common::RandomSource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
                &mut meshes,
                &mut materials,
            ));
        }
        for mut cell in cell_query.iter_mut() {
            cell.age += 1;

            if cell.age >= cell.genes.mature_age {
                cell.mature = true;
            }

            if cell.age >= 100 {
                cell.health = 0.0;
            }
        }
    }
}

fn metabolism_process(mut cell_query: Query<&mut cell::Cell>, time: Res<Time>) {
    for mut cell in cell_query.iter_mut() {
        cell.process_metabolism(time.delta_secs());
    }
}

fn move_cells(
    mut cell_query: Query<(&mut cell::Cell, &mut Transform)>,
    time: Res<Time>,
    game_config: Res<common::GameConfig>,
) {
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cells_snapshot: Vec<cell::Cell> = cell_query.iter().map(|cell| cell.clone()).collect();

    for mut cell in cell_query.iter_mut() {
        let cell_position = Vec2::new(cell.pos_x, cell.pos_y);
        match cell.action {
            Action::Chilling => cell.rest(),
            Action::FindMate => {
                let mut nearest = 100000.0;
                let mut found_mate = false;

                for other_cell in cells_snapshot.iter() {
                    if other_cell.id != cell.id
                        && other_cell.mature
                        && other_cell.action == Action::FindMate
                    {
                        let other_cell_position = Vec2::new(other_cell.pos_x, other_cell.pos_y);

                        if cell.is_within_vision_cone(other_cell_position) {
                            found_mate = true;
                            let distance = other_cell_position.distance(cell_position);

                            if distance < nearest {
                                cell.target_location = Some(other_cell_position);
                                nearest = distance;
                            }

                            if distance < 1.0 {
                                let offspring = cell.create_offspring(
                                    &mut seeded_rng.0,
                                    &game_config,
                                    &mut meshes,
                                    &mut materials,
                                    &other_cell,
                                );
                                commands.spawn(offspring);
                                break;
                            }
                        }
                    }
                }

                if !found_mate {
                    cell.random_target(&mut seeded_rng.0, &game_config);
                }
            }
            Action::GoingForFood => {
                let mut nearest = 100000.0;
                let mut found_food = false;

                for (food, food_entity) in food_query.iter() {
                    let food_position = Vec2::new(food.pos_x, food.pos_y);
                    if cell.is_within_vision_cone(food_position) {
                        found_food = true;
                        let distance = food_position.distance(cell_position);
                        if distance < nearest {
                            cell.target_location = Some(food_position);
                            nearest = distance;
                        }

                        if distance < 1.0 {
                            cell.eat(&food);
                            commands.entity(food_entity).despawn();
                        }
                    }
                }

                if !found_food {
                    cell.random_target(&mut seeded_rng.0, &game_config);
                }
            }
            Action::MovingAround => {
                if cell.target_location.is_none()
                    || cell_position.distance(cell.target_location.unwrap()) < 1.0
                {
                    cell.random_target(&mut seeded_rng.0, &game_config);
                }
            }
        }
    }
}
