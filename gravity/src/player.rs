use std::{cell::RefCell, rc::Rc};

use crate::physics;

use ggegui::{
    egui::{self, RichText},
    Gui,
};
use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawParam},
    GameResult,
};
use physics::{string_to_u128, u128_to_string};
use rand::{random, rngs::ThreadRng, thread_rng};
use rapier2d::prelude::*;

pub struct Player {
    pub physics: Rc<RefCell<physics::Physics>>,
    pub player_cuboid: physics::Cuboid,
    pub is_jumping: bool,
    pub is_falling: bool,
    pub is_moving_left: bool,
    pub is_moving_right: bool,
    pub is_moving_up: bool,
    pub is_moving_down: bool,
    pub is_stopped: bool,
    pub is_grounded: bool,
    pub is_colliding: bool,
    pub player_speed: f32,
    pub jump_speed: f32,
    pub player_width: f32,
    pub player_height: f32,
}

impl Player {
    pub fn new(ctx: &ggez::Context, physics: Rc<RefCell<physics::Physics>>) -> Player {
        let player_cuboid = physics.borrow_mut().new_cuboid(
            ctx,
            physics::string_to_u128("player"),
            400.0,
            100.0,
            0.0,
            0.0,
            Color::RED,
            0.0,
            false,
        );

        Player {
            physics,
            player_cuboid,
            is_jumping: false,
            is_falling: false,
            is_moving_left: false,
            is_moving_right: false,
            is_moving_up: false,
            is_moving_down: false,
            is_stopped: true,
            is_grounded: false,
            is_colliding: false,
            player_speed: 5.0,
            jump_speed: 10.0,
            player_width: 40.0,
            player_height: 100.0,
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.physics
            .borrow_mut()
            .move_to(x, y, self.player_cuboid.collider_handle);
    }
}
