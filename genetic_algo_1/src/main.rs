use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use common::CellSelected;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

mod cell;
mod common;
mod food;
mod helpers;
mod ui;

fn main() {
    let game_config = common::GameConfig {
        movement_cost: 0.05,
        map_height: 3000.0,
        map_width: 3000.0,
        food_spawn_rate: 0.5,
    };

    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, EguiPlugin))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(game_config)
        .insert_resource(common::RandomSource(ChaCha12Rng::from_entropy()))
        .insert_resource(common::CellSelected(None))
        .add_systems(Startup, setup)
        .add_systems(Update, (helpers::camera::movement, game_tick))
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
) {
    for (mut cell_transform, mut cell, entity) in cell_query.iter_mut() {
        // AI stuff
        // For now only random movement
        if cell.target_location.is_none()
            || cell_transform
                .translation
                .truncate()
                .distance(cell.target_location.unwrap())
                < 1.0
        {
            cell.random_target(&mut seeded_rng.0, &game_config);
        }

        // If theres food will get it
        for (food_transform, _, _) in food_query.iter_mut() {
            let food_position = food_transform.translation.truncate();
            if is_within_vision_cone(
                &cell_transform,
                food_position,
                cell.vision_range,
                cell.vision_angle,
            ) {
                cell.action = cell::Action::GoingForFood;
                cell.target_location = Some(food_position);
            }
        }

        // Fixed stuff
        for (food_transform, _, food_entity) in food_query.iter() {
            let distance = cell_transform
                .translation
                .distance(food_transform.translation);
            if distance < 1.0 {
                cell.health += 15.0;
                cell.energy += 15.0;
                commands.entity(food_entity).despawn();
            }
        }

        if let Some(selected_cel) = &cell_selected.0 {
            if selected_cel.id.eq(&cell.id) {
                cell.draw_gismos(&mut gizmos);
            }
        }

        cell.movement(&mut cell_transform, &game_config, time.delta_secs());

        if cell.energy <= 1.0 {
            cell.health -= 1.0;
        }

        if cell.health <= 0.0 {
            commands.entity(entity).despawn();
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

// fn check_collisions(
//     query: Query<(Entity, &Transform, &Mesh2d), With<common::Collider>>,
//     meshes: Res<Assets<Mesh>>,
// ) {
//     let mut colliders = query.iter_combinations();

//     while let Some([(entity_a, transform_a, mesh_a), (entity_b, transform_b, mesh_b)]) =
//         colliders.fetch_next()
//     {
//         let size_a = get_mesh_size(mesh_a, &meshes);
//         let size_b = get_mesh_size(mesh_b, &meshes);

//         if let (Some(size_a), Some(size_b)) = (size_a, size_b) {
//             if aabb_collision(
//                 transform_a.translation.truncate(),
//                 size_a,
//                 transform_b.translation.truncate(),
//                 size_b,
//             ) {

//             }
//         }
//     }
// }

// fn get_mesh_size(mesh_handle: &Mesh2d, meshes: &Assets<Mesh>) -> Option<Vec2> {
//     meshes.get(&mesh_handle.0).map(|mesh| {
//         let aabb = mesh.compute_aabb().unwrap();
//         let size = aabb.half_extents * 2.0;
//         Vec2::new(size.x, size.y)
//     })
// }

// fn aabb_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
//     let half_size1 = size1 / 2.0;
//     let half_size2 = size2 / 2.0;

//     let min1 = pos1 - half_size1;
//     let max1 = pos1 + half_size1;

//     let min2 = pos2 - half_size2;
//     let max2 = pos2 + half_size2;

//     !(min1.x > max2.x || max1.x < min2.x || min1.y > max2.y || max1.y < min2.y)
// }
