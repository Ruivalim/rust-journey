use bevy::prelude::*;
use rand::Rng;

use crate::common;

pub fn cell_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            move_cells,
            cells_die,
            cells_eat,
            cell_reproduction,
            draw_gismos,
            execute_action,
        ),
    )
    .add_systems(FixedUpdate, decide_action);
}
fn move_cells(
    mut cell_query: Query<(&mut Transform, &mut common::Cell)>,
    time: Res<Time>,
    game_config: Res<common::GameConfig>,
) {
    for (mut transform, mut cell) in cell_query.iter_mut() {
        if let Some(target) = cell.target_location {
            let direction = (target - transform.translation.truncate()).normalize();
            let rotation =
                Quat::from_rotation_arc(Vec3::Y, Vec3::new(direction.x, direction.y, 0.0));

            transform.rotation = rotation;

            let dx = direction.x * cell.movement_speed * time.delta_secs();
            let dy = direction.y * cell.movement_speed * time.delta_secs();
            let nx = (transform.translation.x + dx)
                .clamp(-game_config.map_width / 2., game_config.map_width / 2.);
            let ny = (transform.translation.y + dy)
                .clamp(-game_config.map_height / 2., game_config.map_height / 2.);

            transform.translation.x = nx;
            cell.pos_x = nx;

            transform.translation.y = ny;
            cell.pos_y = ny;
            cell.health -= game_config.movement_cost;
            cell.rotation = rotation.to_euler(EulerRot::XYZ).2
        }
    }
}

fn cells_die(
    mut commands: Commands,
    mut cell_query: Query<(&common::Cell, Entity), (Without<common::Food>, With<common::Cell>)>,
) {
    for (cell, entity) in cell_query.iter_mut() {
        if cell.health < 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn cells_eat(
    mut commands: Commands,
    mut cells_query: Query<
        (&mut common::Cell, &Transform),
        (With<common::Cell>, Without<common::Food>),
    >,
    foods_query: Query<(&Transform, Entity), (Without<common::Cell>, With<common::Food>)>,
) {
    for (mut cell, transform) in cells_query.iter_mut() {
        for (food_transform, food_entity) in foods_query.iter() {
            let distance = transform.translation.distance(food_transform.translation);
            if distance < 1.0 {
                cell.health += 15.0;
                commands.entity(food_entity).despawn();
            }
        }
    }
}

fn cell_reproduction(
    mut commands: Commands,
    mut cell_query: Query<(&mut common::Cell, &Transform, Entity)>,
    mut random_source: ResMut<common::RandomSource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rng = &mut random_source.0;

    let cells: Vec<(common::Cell, Transform, Entity)> = cell_query
        .iter()
        .map(|(cell, transform, entity)| (cell.clone(), transform.clone(), entity))
        .collect();

    for (cell1, transform1, entity1) in cells.iter() {
        if cell1.action != common::Action::Reproduce {
            continue;
        }
        for (cell2, transform2, entity2) in cells.iter() {
            if cell1.id == cell2.id || cell1.health < 75.0 || cell2.health < 75.0 {
                continue;
            }

            let distance = transform1.translation.distance(transform2.translation);
            if distance < 50.0 {
                commands.spawn(common::Cell::offspring(
                    rng,
                    &mut meshes,
                    &mut materials,
                    cell1,
                    cell2,
                    transform1,
                    transform2,
                ));

                if let Ok(mut cell1_mut) = cell_query.get_mut(*entity1) {
                    cell1_mut.0.health -= 25.0;
                }
                if let Ok(mut cell2_mut) = cell_query.get_mut(*entity2) {
                    cell2_mut.0.health -= 25.0;
                }
            }
        }
    }
}

fn draw_gismos(selected_cell: Res<common::CellSelected>, mut gizmos: Gizmos) {
    if let Some(cell) = &selected_cell.0 {
        gizmos.circle_2d(
            Vec2::new(cell.pos_x, cell.pos_y),
            cell.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        let forward = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(cell.rotation));
        let half_angle = cell.vision_angle.to_radians() / 2.0;
        let left = forward.rotate(Vec2::from_angle(-half_angle));
        let right = forward.rotate(Vec2::from_angle(half_angle));

        gizmos.line_2d(
            Vec2::new(cell.pos_x, cell.pos_y),
            Vec2::new(cell.pos_x, cell.pos_y) + left * cell.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );
        gizmos.line_2d(
            Vec2::new(cell.pos_x, cell.pos_y),
            Vec2::new(cell.pos_x, cell.pos_y) + right * cell.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        if let Some(location) = cell.target_location {
            gizmos.circle_2d(location, 1.0, Color::linear_rgb(1.0, 0.0, 0.0));
        }
    }
}

fn decide_action(
    mut cell_query: Query<&mut common::Cell>,
    mut random_source: ResMut<common::RandomSource>,
    time: Res<Time>,
) {
    let rng = &mut random_source.0;

    for mut cell in cell_query.iter_mut() {
        cell.action_timer.tick(time.delta());

        if cell.health < 30.0 {
            cell.action = common::Action::FindFood;
        }

        if cell.action_timer.just_finished() {
            cell.action = common::Action::Timeout;
        }

        if cell.action == common::Action::Timeout {
            if cell.health > 75.0 {
                cell.action = common::Action::Reproduce;
            } else {
                if rng.gen() {
                    cell.action = common::Action::FindFood;
                } else {
                    cell.action = common::Action::RandomMovement;
                }
            }

            cell.action_timer = Timer::from_seconds(rng.gen_range(5.0..10.0), TimerMode::Once);
        }
    }
}

fn execute_action(
    mut cell_query: Query<(&mut common::Cell, &Transform)>,
    food_query: Query<&Transform, With<common::Food>>,
    mut random_source: ResMut<common::RandomSource>,
    game_config: Res<common::GameConfig>,
) {
    let rng = &mut random_source.0;

    for (mut cell, transform) in cell_query.iter_mut() {
        match cell.action {
            common::Action::RandomMovement => {
                if cell.target_location.is_none()
                    || transform
                        .translation
                        .truncate()
                        .distance(cell.target_location.unwrap())
                        < 1.0
                {
                    cell.target_location = Some(Vec2::new(
                        rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0),
                        rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0),
                    ));
                }
            }
            common::Action::FindFood => {
                let mut found_food = false;
                for food_transform in food_query.iter() {
                    if is_within_vision_cone(
                        transform,
                        food_transform.translation.truncate(),
                        cell.vision_range,
                        cell.vision_angle,
                    ) {
                        cell.target_location = Some(food_transform.translation.truncate());
                        found_food = true;
                        break;
                    }
                }
                if !found_food {
                    cell.action = common::Action::RandomMovement;
                }
            }
            common::Action::Reproduce => {
                if cell.target_location.is_none()
                    || transform
                        .translation
                        .truncate()
                        .distance(cell.target_location.unwrap())
                        < 1.0
                {
                    cell.target_location = Some(Vec2::new(
                        rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0),
                        rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0),
                    ));
                }
            }
            common::Action::Timeout => {
                cell.target_location = None;
            }
        }
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