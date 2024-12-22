use bevy::prelude::*;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

mod common;
mod game;
mod gameover;
mod menu;

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .init_state::<common::GameState>()
        .add_plugins(game::game_plugin)
        .add_plugins(menu::menu_plugin)
        .add_plugins(gameover::gameover_plugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
