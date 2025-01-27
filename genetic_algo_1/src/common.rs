use bevy::prelude::*;
use rand_chacha::ChaCha12Rng;

#[derive(Resource, Default)]
pub struct CellSelected(pub Option<crate::cell::Cell>);

#[derive(Resource)]
pub struct RandomSource(pub ChaCha12Rng);

pub struct Rewards {
    pub tick_alive: f32,
    pub found_food: f32,
    pub ate_food: f32,
    pub hunger: f32,
    pub reproduction: f32,
}

#[derive(Resource)]
pub struct GameConfig {
    pub map_height: f32,
    pub map_width: f32,
    pub foods_per_day: i32,
    pub current_day: i32,
    pub day_speed: f32,
    pub draw_gizmos: bool,
    pub mutation_rate: f32,
    pub show_fittest: bool,
    pub rewards: Rewards,
}

pub const REWARDS: Rewards = Rewards {
    tick_alive: 0.5,
    found_food: 5.0,
    hunger: -5.0,
    ate_food: 15.0,
    reproduction: 50.0,
};

pub const GAME_CONFIG: GameConfig = GameConfig {
    map_height: 600.0,
    map_width: 800.0,
    foods_per_day: 1,
    current_day: 1,
    day_speed: 1.0,
    draw_gizmos: false,
    mutation_rate: 0.1,
    show_fittest: false,
    rewards: REWARDS,
};

#[derive(Component)]
pub struct Collider;

#[derive(Resource)]
pub struct DayNightCycleTimer(pub Timer);
