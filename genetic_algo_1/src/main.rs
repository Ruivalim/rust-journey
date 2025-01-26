use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_egui::EguiPlugin;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

mod cell;
mod common;
mod helpers;
mod ui;

fn main() {
    let game_config = common::GameConfig {
        movement_cost: 0.05,
        map_height: 800.0,
        map_width: 800.0,
    };

    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<common::MouseCoordinates>()
        .init_resource::<common::CellSelected>()
        .insert_resource(game_config)
        .insert_resource(common::RandomSource(ChaCha12Rng::from_entropy()))
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_plugins(ui::ui_plugin)
        .add_plugins(cell::cell_plugin)
        .add_systems(Update, (helpers::camera::movement,))
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
        let x = seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0);
        let y = seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0);
        commands.spawn((
            common::Food,
            Mesh2d(meshes.add(Rectangle::new(15., 15.))),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
        ));
    }

    for _ in 0..15 {
        let x = seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0);
        let y = seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0);
        let width = seeded_rng.gen_range(15.0..25.0);
        let height = seeded_rng.gen_range(15.0..25.0);
        let family_color = Color::linear_rgb(
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
        );
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(family_color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            common::Cell {
                parent_1: None,
                parent_2: None,
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                width,
                height,
                health: 50.0,
                movement_speed: seeded_rng.gen_range(15.0..100.0),
                vision_range: seeded_rng.gen_range(50.0..200.0),
                attack: 10.0,
                target_location: None,
                offspring_probability: seeded_rng.gen_range(0.1..0.5),
                family_color,
                action: common::Action::RandomMovement,
                action_timer: Timer::from_seconds(seeded_rng.gen_range(1.0..10.0), TimerMode::Once),
            },
        ));
    }

    commands.insert_resource(common::RandomSource(seeded_rng));
}
