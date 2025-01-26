use bevy::prelude::*;
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

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

#[derive(Resource, Default)]
pub struct CellSelected(pub Option<Cell>);

#[derive(Component)]
pub struct Food;

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
}
