use bevy::{input::common_conditions::input_pressed, prelude::*};
use bevy_egui::EguiPlugin;
use common::RandomSource;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

mod cell;
mod common;
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
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<common::MouseCoordinates>()
        .init_resource::<common::CellSelected>()
        .insert_resource(game_config)
        .insert_resource(common::RandomSource(ChaCha12Rng::from_entropy()))
        .insert_resource(common::FoodTimer(Timer::from_seconds(
            10.0,
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_plugins(ui::ui_plugin)
        .add_plugins(cell::cell_plugin)
        .add_systems(Update, new_cell.run_if(input_pressed(KeyCode::KeyN)))
        .add_systems(Update, (helpers::camera::movement, food_spawner))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_config: Res<common::GameConfig>,
) {
    let mut seeded_rng = ChaCha12Rng::from_entropy();

    commands.spawn((Camera2d::default(), common::MainCamera));

    for _ in 0..100 {
        commands.spawn(common::Food::new(
            &mut seeded_rng,
            &game_config,
            &mut meshes,
            &mut materials,
        ));
    }

    for _ in 0..15 {
        commands.spawn(common::Cell::new(
            &mut seeded_rng,
            &game_config,
            &mut meshes,
            &mut materials,
        ));
    }

    commands.insert_resource(common::RandomSource(seeded_rng));
}

fn food_spawner(
    mut commands: Commands,
    mut timer: ResMut<common::FoodTimer>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<RandomSource>,
    game_config: Res<common::GameConfig>,
) {
    let spawn_rate = game_config.food_spawn_rate;

    let interval = (1.0 / spawn_rate).min(0.1);
    timer
        .0
        .set_duration(std::time::Duration::from_secs_f32(interval));

    timer.0.tick(time.delta());
    if timer.0.finished() {
        let num_foods = spawn_rate as u32;
        for _ in 0..num_foods {
            commands.spawn(common::Food::new(
                &mut rng.0,
                &game_config,
                &mut meshes,
                &mut materials,
            ));
        }
    }
}

fn new_cell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_config: Res<common::GameConfig>,
    mut seeded_rng: ResMut<RandomSource>,
) {
    commands.spawn(common::Cell::new(
        &mut seeded_rng.0,
        &game_config,
        &mut meshes,
        &mut materials,
    ));
}
