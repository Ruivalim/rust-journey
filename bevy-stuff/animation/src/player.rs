use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub position: Vec3,
    pub velocity: f32,
}
