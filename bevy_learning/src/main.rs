use bevy::{input::common_conditions::input_pressed, prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContexts, EguiPlugin,
};
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use bevy_fps_controller::controller::*;

#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

#[derive(Component)]
struct Squares;

#[derive(Resource, Default)]
struct MouseCoordinates(Vec2);

#[derive(Component)]
struct MainCamera;

#[derive(Component, Debug)]
struct Square {
    id: f32,
    width: f32,
    height: f32,
    pos_x: f32,
    pos_y: f32,
    dist_x: f32,
    dist_y: f32,
    dir_x: f32,
    dir_y: f32,
    update_time: f32,
    seconds_since_update: f32,
    velocity: f32,
    color: Color,
    z: f32,
}

#[derive(Resource, Default)]
struct SelectedSquare(Option<Square>);

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<MouseCoordinates>()
        .init_resource::<SelectedSquare>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                movement_squares,
                ui_system,
                mouse_coordinates_system,
                get_square_info.run_if(input_pressed(MouseButton::Left)),
            ),
        )
        .add_systems(FixedUpdate, update_squares)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(1231231231);

    commands.spawn((Camera2d::default(), MainCamera));
    let mut rand: rand::prelude::ThreadRng = thread_rng();

    for i in 0..10 {
        let i = i as f32;
        let width = rand.gen_range(10.0..100.0);
        let height = rand.gen_range(10.0..100.0);
        let r = rand.gen_range(0.0..250.0) / 250.0;
        let g = rand.gen_range(0.0..250.0) / 250.0;
        let b = rand.gen_range(0.0..250.0) / 250.0;
        let color = Color::linear_rgb(r, g, b);
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(color))),
            Transform::default().with_translation(Vec3::new(0.0, 0.0, i)),
            Squares,
            Square {
                id: i,
                width,
                height,
                pos_x: 0.0,
                pos_y: 0.0,
                dist_x: 0.0,
                dist_y: 0.0,
                dir_x: 0.0,
                dir_y: 0.0,
                update_time: seeded_rng.gen_range(1.0..5.0),
                seconds_since_update: 0.0,
                velocity: seeded_rng.gen_range(1.0..2.0),
                color,
                z: i,
            },
        ));
    }
    commands.insert_resource(RandomSource(seeded_rng));
}

fn movement_squares(
    mut queries: Query<(&mut Transform, &mut Square), With<Squares>>,
    mut windows: Query<&mut Window>,
    mut selected_square: ResMut<SelectedSquare>,
) {
    let window = windows.single_mut();
    let w_width = window.width() / 2.0;
    let w_height = window.height() / 2.0;

    for mut query in &mut queries {
        let square_data = &mut query.1;
        let translation = &mut query.0.translation;
        let s_width = square_data.width / 2.0;
        let s_height = square_data.height / 2.0;

        if square_data.dist_x > 0.0 {
            translation.x = (translation.x + (square_data.dir_x * square_data.velocity))
                .clamp(-w_width + s_width, w_width - s_width);
            square_data.dist_x -= (1.0 * square_data.velocity).clamp(0.0, square_data.dist_x);
            square_data.pos_x = translation.x;
        }
        if square_data.dist_y > 0.0 {
            translation.y = (translation.y + (square_data.dir_y * square_data.velocity))
                .clamp(-w_height + s_height, w_height - s_height);
            square_data.dist_y -= 1.0 * square_data.velocity.clamp(0.0, square_data.dist_y);
            square_data.pos_y = translation.y;
        }

        if let Some(selected_square) = &mut selected_square.0 {
            if selected_square.id == square_data.id {
                selected_square.pos_x = square_data.pos_x;
                selected_square.pos_y = square_data.pos_y;
                selected_square.dir_x = square_data.dir_x;
                selected_square.dir_y = square_data.dir_y;
                selected_square.dist_x = square_data.dist_x;
                selected_square.dist_y = square_data.dist_y;
                selected_square.seconds_since_update = square_data.seconds_since_update;
            }
        }
    }
}

fn update_squares(
    mut queries: Query<&mut Square, With<Squares>>,
    mut random_source: ResMut<RandomSource>,
    time: Res<Time<Fixed>>,
) {
    let rng = &mut random_source.0;

    for mut query in &mut queries {
        let square_data = &mut query;

        if square_data.seconds_since_update > square_data.update_time {
            if square_data.dist_x <= 0.0 && square_data.dist_y <= 0.0 {
                square_data.dist_x = rng.gen_range(100.0..250.00);
                square_data.dist_y = rng.gen_range(100.0..250.00);
                square_data.dir_x = if rng.gen() { 1.0 } else { -1.0 };
                square_data.dir_y = if rng.gen() { 1.0 } else { -1.0 };
            }

            square_data.seconds_since_update = 0.0;
        } else {
            square_data.seconds_since_update += time.delta_secs();
        }
    }
}

fn ui_system(mut contexts: EguiContexts, selected_square: Res<SelectedSquare>) {
    egui::Window::new("Learning Bevy").show(contexts.ctx_mut(), |ui| {
        ui.label("View Square");
        ui.separator();
        if let Some(square) = &selected_square.0 {
            ui.label(format!("Id: {}", square.id));
            ui.label(format!("Width: {}", square.width));
            ui.label(format!("Height: {}", square.height));
            ui.label(format!("Position X: {}", square.pos_x));
            ui.label(format!("Position Y: {}", square.pos_y));
            ui.label(format!("Distance X: {}", square.dist_x));
            ui.label(format!("Distance Y: {}", square.dist_y));
            ui.label(format!("Direction X: {}", square.dir_x));
            ui.label(format!("Direction Y: {}", square.dir_y));
            ui.label(format!("Update Time: {}", square.update_time));
            ui.label(format!(
                "Seconds Since Update: {}",
                square.seconds_since_update
            ));
            ui.label(format!("Velocity: {}", square.velocity));
            ui.label(
                RichText::new(format!(
                    "Color: {},{},{}",
                    square.color.to_linear().red * 255.0,
                    square.color.to_linear().blue * 255.0,
                    square.color.to_linear().green * 255.0,
                ))
                .background_color(Color32::from_hex(&square.color.to_srgba().to_hex()).unwrap()),
            );
        } else {
            ui.label("No square selected");
        }
    });
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

fn get_square_info(
    mut selected_square: ResMut<SelectedSquare>,
    queries: Query<&Square, With<Squares>>,
    mouse_coordinates: ResMut<MouseCoordinates>,
) {
    let m_x = mouse_coordinates.0.x;
    let m_y = mouse_coordinates.0.y;

    let squares_on_position = queries.iter().filter(|square| {
        let s_width = square.width / 2.0;
        let s_height = square.height / 2.0;
        let x = square.pos_x;
        let y = square.pos_y;
        let x_min = x - s_width;
        let x_max = x + s_width;
        let y_min = y - s_height;
        let y_max = y + s_height;

        m_x >= x_min && m_x <= x_max && m_y >= y_min && m_y <= y_max
    });

    let squares_on_position = squares_on_position.collect::<Vec<&Square>>();
    let count = squares_on_position.len();

    if count == 0 as usize {
        selected_square.0 = None;
        return;
    }

    let square_on_front = squares_on_position
        .iter()
        .fold(None as Option<&Square>, |acc, square| {
            if let Some(acc) = acc {
                if acc.z > square.z {
                    Some(acc)
                } else {
                    Some(square)
                }
            } else {
                Some(square)
            }
        })
        .unwrap()
        .to_owned();

    selected_square.0 = Some(Square {
        id: square_on_front.id,
        width: square_on_front.width,
        height: square_on_front.height,
        pos_x: square_on_front.pos_x,
        pos_y: square_on_front.pos_y,
        dist_x: square_on_front.dist_x,
        dist_y: square_on_front.dist_y,
        dir_x: square_on_front.dir_x,
        dir_y: square_on_front.dir_y,
        update_time: square_on_front.update_time,
        seconds_since_update: square_on_front.seconds_since_update,
        velocity: square_on_front.velocity,
        color: square_on_front.color,
        z: square_on_front.z,
    });
}
