use std::process::id;

use bevy::{
    color::palettes::css::{GREEN, RED},
    input::common_conditions::input_pressed,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{EguiContexts, EguiPlugin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

mod helpers;

const MOVEMENT_COST: f32 = 0.05;
const ATTACK_COST: f32 = 15.;
const MAP_HEIGHT: f32 = 1500.0;
const MAP_WIDTH: f32 = 1500.0;

#[derive(Resource)]
pub struct RandomSource(ChaCha12Rng);

#[derive(Component, Clone, Copy)]
pub struct Cell {
    id: Uuid,
    pos_x: f32,
    pos_y: f32,
    width: f32,
    height: f32,
    movement_speed: f32,
    vision_range: f32,
    health: f32,
    attack: f32,
    target_location: Option<Vec2>,
    movement_probability: f64,
}

#[derive(Resource, Default)]
pub struct CellSelected(Option<Cell>);

#[derive(Component)]
pub struct Food;

#[derive(Resource, Default)]
struct MouseCoordinates(Vec2);

#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<MouseCoordinates>()
        .init_resource::<CellSelected>()
        .insert_resource(RandomSource(ChaCha12Rng::from_entropy()))
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                helpers::camera::movement,
                move_cells,
                cells_die,
                ui_system,
                mouse_coordinates_system,
                get_cell_info.run_if(input_pressed(MouseButton::Left)),
                update_cell_info,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut seeded_rng = ChaCha12Rng::from_entropy();

    commands.spawn((Camera2d::default(), MainCamera));

    for _ in 0..30 {
        let x = seeded_rng.gen_range(-MAP_WIDTH / 2.0..MAP_WIDTH / 2.0);
        let y = seeded_rng.gen_range(-MAP_HEIGHT / 2.0..MAP_HEIGHT / 2.0);
        commands.spawn((
            Food,
            Mesh2d(meshes.add(Rectangle::new(15., 15.))),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
        ));
    }

    for _ in 0..5 {
        let x = seeded_rng.gen_range(-MAP_WIDTH / 2.0..MAP_WIDTH / 2.0);
        let y = seeded_rng.gen_range(-MAP_HEIGHT / 2.0..MAP_HEIGHT / 2.0);
        let width = seeded_rng.gen_range(15.0..25.0);
        let height = seeded_rng.gen_range(15.0..25.0);
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            Cell {
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                width,
                height,
                health: 50.0,
                movement_speed: seeded_rng.gen_range(0.15..100.0),
                vision_range: seeded_rng.gen_range(0.15..100.0),
                attack: 10.0,
                target_location: None,
                movement_probability: seeded_rng.gen(),
            },
        ));
    }

    commands.insert_resource(RandomSource(seeded_rng));
}

fn move_cells(
    mut cell_query: Query<(&mut Transform, &mut Cell), (Without<Food>, With<Cell>)>,
    food_query: Query<&Transform, (With<Food>, Without<Cell>)>,
    time: Res<Time>,
    mut random_source: ResMut<RandomSource>,
) {
    let rng = &mut random_source.0; // Use the random source directly
    for (mut transform, mut cell) in cell_query.iter_mut() {
        if rng.gen_bool(cell.movement_probability) {
            continue;
        }

        if cell.target_location.is_none()
            || transform
                .translation
                .truncate()
                .distance(cell.target_location.unwrap())
                < 1.0
        {
            cell.target_location = Some(Vec2::new(
                rng.gen_range(-MAP_WIDTH / 2.0..MAP_WIDTH / 2.0),
                rng.gen_range(-MAP_HEIGHT / 2.0..MAP_HEIGHT / 2.0),
            ));
        }

        for food_transform in food_query.iter() {
            let distance = transform
                .translation
                .truncate()
                .distance(food_transform.translation.truncate());
            if distance <= cell.vision_range {
                cell.target_location = Some(food_transform.translation.truncate());
                break;
            }
        }

        if let Some(target) = cell.target_location {
            let direction = (target - transform.translation.truncate()).normalize();
            let dx = direction.x * cell.movement_speed * time.delta_secs();
            let dy = direction.y * cell.movement_speed * time.delta_secs();
            let nx = (transform.translation.x + dx).clamp(-MAP_WIDTH / 2., MAP_WIDTH / 2.);
            let ny = (transform.translation.x + dy).clamp(-MAP_WIDTH / 2., MAP_WIDTH / 2.);

            transform.translation.x = nx;
            cell.pos_x = nx;

            transform.translation.y = ny;
            cell.pos_y = ny;
            cell.health -= MOVEMENT_COST;
        }
    }
}

fn cells_die(
    mut commands: Commands,
    mut cell_query: Query<(&Cell, Entity), (Without<Food>, With<Cell>)>,
) {
    for (cell, entity) in cell_query.iter_mut() {
        if cell.health < 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn mouse_coordinates_system(
    mut mouse_coordinates: ResMut<MouseCoordinates>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        mouse_coordinates.0 = world_position;
    }
}

fn ui_system(mut contexts: EguiContexts, selected_cell: Res<CellSelected>) {
    egui::Window::new("Cell Viewer").show(contexts.ctx_mut(), |ui| {
        ui.separator();
        if let Some(cell) = &selected_cell.0 {
            ui.label(format!("ID: {}", cell.id.to_string()));
            ui.label(format!("Width: {}", cell.width));
            ui.label(format!("Height: {}", cell.height));
            ui.label(format!("Health: {}", cell.health));
            ui.label(format!("Pos X: {}", cell.pos_x));
            ui.label(format!("Pox Y: {}", cell.pos_y));
            ui.label(format!("Target: {:?}", cell.target_location));
            ui.label(format!("Speed: {}", cell.movement_speed));
            ui.label(format!(
                "Probability Movement: {}",
                cell.movement_probability
            ));
        } else {
            ui.label("No cell selected");
        }
    });
}

fn get_cell_info(
    mut selected_cell: ResMut<CellSelected>,
    queries: Query<(&Cell), With<Cell>>,
    mouse_coordinates: ResMut<MouseCoordinates>,
) {
    let m_x = mouse_coordinates.0.x;
    let m_y = mouse_coordinates.0.y;

    let cells_on_position = queries.iter().filter(|cell| {
        let s_width = cell.width / 2.0;
        let s_height = cell.height / 2.0;
        let x = cell.pos_x;
        let y = cell.pos_y;
        let x_min = x - s_width;
        let x_max = x + s_width;
        let y_min = y - s_height;
        let y_max = y + s_height;

        m_x >= x_min && m_x <= x_max && m_y >= y_min && m_y <= y_max
    });

    let cells_on_position = cells_on_position.collect::<Vec<&Cell>>();
    let count = cells_on_position.len();

    if count == 0 as usize {
        selected_cell.0 = None;
        return;
    }

    selected_cell.0 = Some(cells_on_position[0].clone());
}

fn update_cell_info(mut selected_cell: ResMut<CellSelected>, queries: Query<(&Cell), With<Cell>>) {
    if let Some(cell_sel) = selected_cell.0 {
        let cell_selected = queries.iter().filter(|cell| cell.id.eq(&cell_sel.id));
        let cell_selected = cell_selected.collect::<Vec<&Cell>>();
        let count = cell_selected.len();

        if count == 0 as usize {
            selected_cell.0 = None;
            return;
        }

        selected_cell.0 = Some(cell_selected[0].clone());
    }
}
