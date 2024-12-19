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
    pub player_body: physics::Cuboid,
    pub player_speed: f32,
    pub jump_speed: f32,
    pub player_width: f32,
    pub player_height: f32,
    pub is_jumping: bool,
    pub is_grounded: bool,
}

impl Player {
    pub fn new(
        ctx: &ggez::Context,
        physics: Rc<RefCell<physics::Physics>>,
        width: f32,
        height: f32,
        speed: f32,
        jump: f32,
    ) -> Player {
        let player_body = physics.borrow_mut().new_cuboid(
            ctx,
            physics::string_to_u128("player"),
            width,
            height,
            0.0,
            0.0,
            Color::RED,
            1.0,
            false,
        );

        // speed calculated taking the players mass into account
        let mass = physics
            .borrow()
            .rigid_body_set
            .get(player_body.body_handle)
            .unwrap()
            .mass();
        let player_speed = speed * mass;
        let jump_speed = jump * mass;

        Player {
            physics,
            player_body,
            player_speed,
            jump_speed,
            player_width: width,
            player_height: height,
            is_jumping: false,
            is_grounded: false,
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.physics
            .borrow_mut()
            .move_to(x, y, self.player_body.collider_handle);
    }

    pub fn player_movement(&mut self, key: ggez::input::keyboard::KeyCode) {
        let mut force = Vector::new(0.0, 0.0);

        match key {
            ggez::input::keyboard::KeyCode::A => {
                if !self.is_jumping {
                    force.x = -self.player_speed;
                }
            }
            ggez::input::keyboard::KeyCode::D => {
                if !self.is_jumping {
                    force.x = self.player_speed;
                }
            }
            ggez::input::keyboard::KeyCode::W => {
                if !self.is_jumping {
                    force.y = -self.jump_speed;
                    self.is_jumping = true;
                }
            }
            _ => {}
        }

        self.physics
            .borrow_mut()
            .apply_impulse(self.player_body.body_handle, force);
    }

    pub fn update(&mut self) {
        let physics = self.physics.borrow();
        let body = physics
            .rigid_body_set
            .get(self.player_body.body_handle)
            .unwrap();
        let velocity = body.linvel();
        println!("velocity: {:?}", velocity.y.abs());
        if velocity.y.abs() < 0.1 {
            self.is_jumping = false;
            self.is_grounded = true;
        }
    }
}
