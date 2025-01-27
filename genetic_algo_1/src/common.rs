use bevy::{color::palettes::css::GREEN, prelude::*};
use rand::Rng;
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

const BASE_HEALTH: f32 = 50.0;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum Action {
    FindFood,
    Reproduce,
    RandomMovement,
    Timeout,
}

#[derive(Component, Clone)]
pub struct Cell {
    pub parent_1: Option<Uuid>,
    pub parent_2: Option<Uuid>,
    pub id: Uuid,
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
    pub movement_speed: f32,
    pub vision_range: f32,
    pub health: f32,
    pub attack: f32,
    pub target_location: Option<Vec2>,
    pub offspring_probability: f64,
    pub family_color: Color,
    pub action: Action,
    pub action_timer: Timer,
    pub vision_angle: f32,
    pub rotation: f32,
}

impl Cell {
    pub fn new(
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        Cell,
    ) {
        let x = seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0);
        let y = seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0);
        let width = seeded_rng.gen_range(15.0..25.0);
        let height = seeded_rng.gen_range(15.0..25.0);
        let family_color = Color::linear_rgb(
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
        );

        return (
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(family_color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            Cell {
                parent_1: None,
                parent_2: None,
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                width,
                height,
                health: BASE_HEALTH,
                movement_speed: seeded_rng.gen_range(15.0..100.0),
                vision_range: seeded_rng.gen_range(50.0..200.0),
                attack: 10.0,
                target_location: None,
                offspring_probability: seeded_rng.gen_range(0.1..0.5),
                family_color,
                action: Action::RandomMovement,
                action_timer: Timer::from_seconds(seeded_rng.gen_range(1.0..10.0), TimerMode::Once),
                vision_angle: 90.0,
                rotation: 0.0,
            },
        );
    }

    pub fn offspring(
        seeded_rng: &mut ChaCha12Rng,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        cell1: &Cell,
        cell2: &Cell,
        transform1: &Transform,
        transform2: &Transform,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        Cell,
    ) {
        let offspring = Cell {
            parent_1: Some(cell1.id),
            parent_2: Some(cell2.id),
            id: Uuid::new_v4(),
            pos_x: (transform1.translation.x + transform2.translation.x) / 2.0,
            pos_y: (transform1.translation.y + transform2.translation.y) / 2.0,
            width: (cell1.width + cell2.width) / 2.0 * seeded_rng.gen_range(0.9..1.1),
            height: (cell1.height + cell2.height) / 2.0 * seeded_rng.gen_range(0.9..1.1),
            movement_speed: (cell1.movement_speed + cell2.movement_speed) / 2.0
                * seeded_rng.gen_range(0.9..1.1),
            vision_range: (cell1.vision_range + cell2.vision_range) / 2.0
                * seeded_rng.gen_range(0.9..1.1),
            health: BASE_HEALTH,
            attack: (cell1.attack + cell2.attack) / 2.0 * seeded_rng.gen_range(0.9..1.1),
            target_location: None,
            offspring_probability: (cell1.offspring_probability + cell2.offspring_probability)
                / 2.0
                * seeded_rng.gen_range(0.9..1.1),
            family_color: blend_colors(
                cell1.family_color.to_linear(),
                cell2.family_color.to_linear(),
                seeded_rng,
            ),
            action: Action::RandomMovement,
            action_timer: Timer::from_seconds(seeded_rng.gen_range(5.0..10.0), TimerMode::Once),
            vision_angle: 90.0,
            rotation: 0.0,
        };

        return (
            Mesh2d(meshes.add(Rectangle::new(offspring.width, offspring.height))),
            MeshMaterial2d(materials.add(offspring.family_color)),
            Transform::default().with_translation(Vec3::new(offspring.pos_x, offspring.pos_y, 0.0)),
            offspring,
        );
    }
}

#[derive(Resource, Default)]
pub struct CellSelected(pub Option<Cell>);

#[derive(Component)]
pub struct Food;

impl Food {
    pub fn new(
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> (
        Food,
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
    ) {
        let x = seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0);
        let y = seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0);

        return (
            Food,
            Mesh2d(meshes.add(Rectangle::new(15., 15.))),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
        );
    }
}

#[derive(Resource, Default)]
pub struct MouseCoordinates(pub Vec2);

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct RandomSource(pub ChaCha12Rng);

#[derive(Resource)]
pub struct GameConfig {
    pub movement_cost: f32,
    pub map_height: f32,
    pub map_width: f32,
    pub food_spawn_rate: f32,
}

#[derive(Resource)]
pub struct FoodTimer(pub Timer);

pub fn blend_colors(color1: LinearRgba, color2: LinearRgba, rng: &mut ChaCha12Rng) -> Color {
    let r = (color1.red + color2.red) / 2.0 * rng.gen_range(0.9..1.1);
    let g = (color1.green + color2.green) / 2.0 * rng.gen_range(0.9..1.1);
    let b = (color1.blue + color2.blue) / 2.0 * rng.gen_range(0.9..1.1);

    Color::linear_rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}
