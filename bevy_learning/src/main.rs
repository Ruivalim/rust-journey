use bevy::{
    asset::RenderAssetUsages,
    color::palettes::{basic::PURPLE, css},
    log::tracing_subscriber::field::debug,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::hashbrown::hash_map::IterMut,
};
use rand::{rngs::ThreadRng, thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use bevy_fps_controller::controller::*;

#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

#[derive(Component)]
struct Squares;

#[derive(Component, Debug)]
struct Square {
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
}

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, movement_squares)
        .add_systems(FixedUpdate, update_squares)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(1231231231);

    commands.spawn(Camera2d);
    let mut rand: rand::prelude::ThreadRng = thread_rng();

    for i in 0..1000 {
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
                velocity: seeded_rng.gen_range(1.0..10.0),
            },
        ));
    }
    commands.insert_resource(RandomSource(seeded_rng));
}

fn movement_squares(
    mut queries: Query<(&mut Transform, &mut Square), With<Squares>>,
    mut windows: Query<&mut Window>,
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
            square_data.dist_x -= 1.0 * square_data.velocity;
            square_data.pos_x = translation.x;
        }
        if square_data.dist_y > 0.0 {
            translation.y = (translation.y + (square_data.dir_y * square_data.velocity))
                .clamp(-w_height + s_height, w_height - s_height);
            square_data.dist_y -= 1.0 * square_data.velocity;
            square_data.pos_y = translation.y;
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
                square_data.dist_x = rng.gen_range(0.0..100.00);
                square_data.dist_y = rng.gen_range(0.0..100.00);
                square_data.dir_x = if rng.gen() { 1.0 } else { -1.0 };
                square_data.dir_y = if rng.gen() { 1.0 } else { -1.0 };
            }

            square_data.seconds_since_update = 0.0;
        } else {
            square_data.seconds_since_update += time.delta_secs();
        }
    }
}
