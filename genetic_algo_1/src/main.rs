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
        .add_systems(Update, (helpers::camera::movement, game_tick))
        .add_systems(FixedUpdate, (day_night_cycle, metabolism_process))
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
    mut commands: Commands,
    time: Res<Time>,
    game_config: Res<common::GameConfig>,
    mut cell_query: Query<
        (&mut Transform, &mut cell::Cell, Entity),
        (With<cell::Cell>, Without<food::Food>),
    >,
    mut food_query: Query<
        (&mut Transform, &mut food::Food, Entity),
        (With<food::Food>, Without<cell::Cell>),
    >,
    mut seeded_rng: ResMut<common::RandomSource>,
    cell_selected: Res<CellSelected>,
    mut gizmos: Gizmos,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let query_snapshot: Vec<(Transform, cell::Cell, Entity)> = cell_query
        .iter()
        .map(|(transform, cell, entity)| (transform.clone(), cell.clone(), entity))
        .collect();

    for (mut cell_transform, mut cell, entity) in cell_query.iter_mut() {
        let mut reward = 0.0;
        if cell.health <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        cell.process_brain(&mut seeded_rng.0);

        reward += game_config.rewards.tick_alive;

        match cell.action {
            Action::Chilling => {
                cell.rest();
            }
            Action::GoingForFood => {
                let mut food_found = false;

                for (food_transform, _, _) in food_query.iter_mut() {
                    let food_position = food_transform.translation.truncate();
                    if is_within_vision_cone(
                        &cell_transform,
                        food_position,
                        cell.genes.vision_range,
                        cell.genes.vision_angle,
                    ) {
                        reward += game_config.rewards.found_food;
                        cell.target_location = Some(food_position);
                        food_found = true;
                        break;
                    }
                }

                if !food_found {
                    if cell.target_location.is_none()
                        || cell_transform
                            .translation
                            .truncate()
                            .distance(cell.target_location.unwrap())
                            < 1.0
                    {
                        cell.random_target(&mut seeded_rng.0, &game_config);
                    }
                }
            }
            Action::MovingAround => {
                if cell.target_location.is_none()
                    || cell_transform
                        .translation
                        .truncate()
                        .distance(cell.target_location.unwrap())
                        < 1.0
                {
                    cell.random_target(&mut seeded_rng.0, &game_config);
                }
            }

            Action::FindMate => {
                let mut mate: Option<cell::Cell> = None;
                // TODO: cone based vision to find, like the food
                for (other_transform, other_cell, _) in query_snapshot.iter() {
                    let distance = cell_transform
                        .translation
                        .truncate()
                        .distance(other_transform.translation.truncate());
                    if distance < 50.0
                        && other_cell.action != Action::FindMate
                        && !cell.id.eq(&other_cell.id)
                        && other_cell.energy > 70.0
                        && cell.energy > 70.0
                    {
                        mate = Some(other_cell.clone());
                    }
                }

                if let Some(mate_cell) = mate {
                    cell.target_location = Some(Vec2::new(mate_cell.pos_x, mate_cell.pos_y));

                    if cell_transform
                        .translation
                        .truncate()
                        .distance(Vec2::new(mate_cell.pos_x, mate_cell.pos_y))
                        < 10.0
                    {
                        let offspring = cell::Cell::create_offspring(
                            &mut seeded_rng.0,
                            &game_config,
                            &mut meshes,
                            &mut materials,
                            cell.clone(),
                            mate_cell,
                        );

                        commands.spawn(offspring);
                        cell.energy -= cell.genes.birth_energy_loss;
                        reward += game_config.rewards.reproduction;
                    }
                } else {
                    if cell.target_location.is_none()
                        || cell_transform
                            .translation
                            .truncate()
                            .distance(cell.target_location.unwrap())
                            < 1.0
                    {
                        cell.random_target(&mut seeded_rng.0, &game_config);
                    }
                }
            }
        }

        if cell.hunger >= 100.0 {
            reward -= game_config.rewards.hunger;
        }

        for (food_transform, _, food_entity) in food_query.iter() {
            let distance = cell_transform
                .translation
                .distance(food_transform.translation);
            if distance < 1.0 {
                // TODO: Change food properties
                cell.energy = (cell.energy + 15.0).clamp(0.0, 100.0);
                cell.health = (cell.energy + 15.0).clamp(0.0, 100.0);
                cell.hunger = (cell.hunger - 15.0).clamp(0.0, 100.0);
                commands.entity(food_entity).despawn();
                reward += game_config.rewards.ate_food;
            }
        }

        if let Some(selected_cel) = &cell_selected.0 {
            if selected_cel.id.eq(&cell.id) {
                cell.draw_vision(&mut gizmos);
            }
        }

        if game_config.draw_gizmos {
            cell.draw_gizmos(&mut gizmos);
        }

        cell.movement(&mut cell_transform, &game_config, time.delta_secs());
        cell.fitness += reward;
    }
}

fn is_within_vision_cone(
    cell_transform: &Transform,
    target_position: Vec2,
    vision_range: f32,
    vision_angle: f32,
) -> bool {
    let direction_to_target = (target_position - cell_transform.translation.truncate()).normalize();
    let cell_forward = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(
        cell_transform.rotation.to_euler(EulerRot::XYZ).2,
    ));

    let distance = cell_transform
        .translation
        .truncate()
        .distance(target_position);
    if distance > vision_range {
        return false;
    }

    let angle_to_target = cell_forward.angle_to(direction_to_target).to_degrees();
    angle_to_target.abs() <= vision_angle / 2.0
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
        }
    }
}

fn metabolism_process(mut cell_query: Query<&mut cell::Cell>, time: Res<Time>) {
    for mut cell in cell_query.iter_mut() {
        cell.process_metabolism(time.delta_secs());
    }
}
