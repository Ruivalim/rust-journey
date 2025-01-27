use bevy::prelude::*;
use rand_chacha::ChaCha12Rng;

#[derive(Resource, Default)]
pub struct CellSelected(pub Option<crate::cell::Cell>);

#[derive(Resource)]
pub struct RandomSource(pub ChaCha12Rng);

#[derive(Resource)]
pub struct GameConfig {
    pub movement_cost: f32,
    pub map_height: f32,
    pub map_width: f32,
    pub foods_per_day: i32,
    pub hunger_over_time: f32,
    pub life_lost_on_hungry: f32,
    pub current_day: i32,
    pub day_speed: f32,
    pub draw_gizmos: bool,
    pub mutation_rate: f32,
    pub show_fittest: bool,
}

#[derive(Component)]
pub struct Collider;

#[derive(Resource)]
pub struct DayNightCycleTimer(pub Timer);
