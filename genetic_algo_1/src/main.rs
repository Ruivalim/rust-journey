use bevy::{
    color::palettes::css::{GREEN, RED},
    prelude::*,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

mod helpers;

const MOVEMENT_COST: f32 = 5.;
const ATTACK_COST: f32 = 15.;
const MAP_HEIGHT: f32 = 1500.0;
const MAP_WIDTH: f32 = 1500.0;

#[derive(Resource)]
pub struct RandomSource(ChaCha12Rng);

#[derive(Component)]
pub struct Cell {
    width: f32,
    height: f32,
    movement_speed: f32,
    //vision_size: f32,
    health: f32,
    attack: f32,
}

#[derive(Component)]
pub struct Food;

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (helpers::camera::movement))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut seeded_rng = ChaCha12Rng::from_entropy();

    commands.spawn(Camera2d);

    for _ in 0..30 {
        let x = seeded_rng.gen_range(-1024.0..1024.0);
        let y = seeded_rng.gen_range(-1024.0..1024.0);
        commands.spawn((
            Food,
            Mesh2d(meshes.add(Rectangle::new(15., 15.))),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
        ));
    }

    for _ in 0..5 {
        let x = seeded_rng.gen_range(-1024.0..1024.0);
        let y = seeded_rng.gen_range(-1024.0..1024.0);
        let width = seeded_rng.gen_range(15.0..25.0);
        let height = seeded_rng.gen_range(15.0..25.0);
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            Cell {
                width,
                height,
                health: 50.0,
                movement_speed: seeded_rng.gen_range(0.0..25.0),
                attack: 10.0,
            },
        ));
    }

    commands.insert_resource(RandomSource(seeded_rng));
}
