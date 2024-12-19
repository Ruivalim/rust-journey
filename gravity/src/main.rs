mod physics;

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

struct MainState {
    balls: Vec<physics::Ball>,
    floors: Vec<physics::Cuboid>,
    physics: physics::Physics,
    gui: Gui,
    rng: ThreadRng,
    open_ui: bool,
    is_mouse_left_down: bool,
    last_clicked_collider: Option<ColliderHandle>,
}

impl MainState {
    fn new(ctx: &ggez::Context, canvas_width: f32, canvas_height: f32) -> GameResult<MainState> {
        let mut physics = physics::Physics::new(Vector::new(0.0, 98.1));
        physics.integration_parameters.max_ccd_substeps = 10;

        let flor_width = canvas_width;
        let flor_height = 100.0;

        let mut floors = vec![];

        floors.push(physics.new_cuboid(
            ctx,
            string_to_u128("floor"),
            flor_width,
            flor_height,
            0.0,
            canvas_height - flor_height,
            Color::WHITE,
            0.7,
            true,
        ));

        floors.push(physics.new_cuboid(
            ctx,
            string_to_u128("ceeiling"),
            flor_width,
            flor_height,
            0.0,
            0.0,
            Color::WHITE,
            0.7,
            true,
        ));

        floors.push(physics.new_cuboid(
            ctx,
            string_to_u128("left_wall"),
            flor_height,
            canvas_height,
            0.0,
            0.0,
            Color::WHITE,
            0.7,
            true,
        ));

        floors.push(physics.new_cuboid(
            ctx,
            string_to_u128("right_wall"),
            flor_height,
            canvas_height,
            canvas_width - flor_height,
            0.0,
            Color::WHITE,
            0.7,
            true,
        ));

        let mut balls = vec![];

        for i in 0..100 {
            let ball_init_x = random::<f32>() * canvas_width;
            let ball_init_y = random::<f32>() * canvas_height;
            let ball = physics.new_ball(
                ctx,
                string_to_u128(format!("ball_{}", i).as_str()),
                ball_init_x.clamp(120.0, canvas_width - 120.0),
                ball_init_y.clamp(120.0, canvas_height - 120.0),
                20.0,
                Color::WHITE,
                0.7,
                false,
            );
            balls.push(ball);
        }

        let gui = Gui::new(ctx);

        Ok(MainState {
            balls,
            floors,
            physics,
            gui,
            rng: thread_rng(),
            open_ui: true,
            is_mouse_left_down: false,
            last_clicked_collider: None,
        })
    }

    fn draw_gizmos(&self, ctx: &ggez::Context, canvas: &mut Canvas) {
        for gizmo in self.physics.render_gizmos(ctx) {
            canvas.draw(&gizmo, DrawParam::default());
        }
    }
}

impl EventHandler for MainState {
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.is_mouse_left_down = true;
        let query = self.physics.query_point(Vector::new(_x, _y));
        for collider_handle in query {
            self.last_clicked_collider = Some(collider_handle);
            let body_handle = self.physics.collider_set[collider_handle].parent().unwrap();
            let user_data = self.physics.rigid_body_set[body_handle].user_data;
            println!("User data: {:?}", u128_to_string(user_data));
        }

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.is_mouse_left_down = false;
        self.last_clicked_collider = None;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        if input.keycode == Some(ggez::input::keyboard::KeyCode::Space) {
            self.open_ui = !self.open_ui;
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut ggez::Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        if self.is_mouse_left_down {
            if let Some(collider_handle) = self.last_clicked_collider {
                let body_handle = self.physics.collider_set[collider_handle].parent().unwrap();
                let body = self.physics.rigid_body_set.get_mut(body_handle).unwrap();

                let current_pos = body.position().translation.vector;
                let target_pos = Vector::new(x, y);

                let force = target_pos - current_pos;

                body.apply_impulse(force * 150.0, true);
            }
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> GameResult {
        self.physics.step();
        if self.open_ui {
            let gui_ctx = self.gui.ctx();

            egui::Window::new("UI").show(&gui_ctx, |ui| {
                if ui.button(RichText::new("EXPLODE!!").size(40.0)).clicked() {
                    for ball in &self.balls {
                        let force = 1000000.0;
                        let rand_x = (random::<f32>() * force) * if random() { 1.0 } else { -1.0 };
                        let rand_y = (random::<f32>() * force) * if random() { 1.0 } else { -1.0 };
                        self.physics
                            .apply_impulse(ball.body_handle, Vector::new(rand_x, rand_y));
                    }
                }
                if ui.button(RichText::new("New Ball").size(40.0)).clicked() {
                    let ball_init_x = random::<f32>() * 2000.0;
                    let ball_init_y = random::<f32>() * 1000.0;
                    let ball = self.physics.new_ball(
                        ctx,
                        string_to_u128(format!("ball_{}", self.balls.len() + 1).as_str()),
                        ball_init_x.clamp(120.0, 2000.0 - 120.0),
                        ball_init_y.clamp(120.0, 1000.0 - 120.0),
                        20.0,
                        Color::WHITE,
                        0.7,
                        false,
                    );
                    self.balls.push(ball);
                }
                if ui.button(RichText::new("Quit").size(40.0)).clicked() {
                    ctx.request_quit();
                }
                if ui.button(RichText::new("Close UI").size(40.0)).clicked() {
                    self.open_ui = false;
                }
            });
            self.gui.update(ctx);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::CYAN);

        for floor in &self.floors {
            let floor_body = &self.physics.rigid_body_set[floor.body_handle].translation();
            let x = floor_body.x - floor.size.x / 2.0;
            let y = floor_body.y - floor.size.y / 2.0;
            canvas.draw(&floor.mesh, DrawParam::default().dest(Vec2::new(x, y)));
        }

        for ball in &self.balls {
            let ball_body = &self.physics.rigid_body_set[ball.body_handle].translation();
            canvas.draw(
                &ball.mesh,
                DrawParam::default().dest(Vec2::new(ball_body.x, ball_body.y)),
            );
        }

        self.draw_gizmos(&ctx, &mut canvas);

        canvas.draw(&self.gui, DrawParam::default().dest(Vec2::ZERO));

        canvas.finish(ctx)
    }
}

fn main() {
    let width = 2000.0;
    let height = 1000.0;
    let context_builder = ggez::ContextBuilder::new("Playing", "Rui")
        .window_mode(WindowMode::default().dimensions(width, height));
    let (ctx, event_loop) = context_builder.build().unwrap();

    let state = MainState::new(&ctx, width, height).unwrap();
    event::run(ctx, event_loop, state)
}
