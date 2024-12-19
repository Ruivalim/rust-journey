mod physics;

use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawParam, Mesh},
    GameResult,
};
use rand::random;
use rapier2d::prelude::*;

struct MainState {
    balls: Vec<(Mesh, RigidBodyHandle)>,
    floors: Vec<(Mesh, RigidBodyHandle, Vec2)>,
    physics: physics::Physics,
}

impl MainState {
    fn new(ctx: &ggez::Context, canvas_width: f32, canvas_height: f32) -> GameResult<MainState> {
        let mut physics = physics::Physics::new(Vector::new(0.0, 98.1));

        let mut balls = vec![];

        for _ in 0..100 {
            let ball_init_x = random::<f32>() * canvas_width;
            let ball_init_y = random::<f32>() * canvas_height;
            let ball = physics.new_ball(
                ctx,
                ball_init_x.clamp(120.0, canvas_width - 120.0),
                ball_init_y.clamp(120.0, canvas_height - 120.0),
                20.0,
                Color::WHITE,
                0.7,
                false,
            );
            balls.push(ball);
        }

        let flor_width = canvas_width;
        let flor_height = 100.0;

        let mut floors = vec![];

        floors.push(physics.new_cuboid(
            ctx,
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
            flor_height,
            canvas_height,
            canvas_width - flor_height,
            0.0,
            Color::WHITE,
            0.7,
            true,
        ));

        Ok(MainState {
            balls,
            floors,
            physics,
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
        let new_floor =
            self.physics
                .new_cuboid(_ctx, 100.0, 100.0, _x, _y, Color::WHITE, 0.7, true);

        self.floors.push(new_floor);

        Ok(())
    }

    fn update(&mut self, _ctx: &mut ggez::Context) -> GameResult {
        self.physics.step();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::CYAN);

        for ball in &self.balls {
            let ball_body = &self.physics.rigid_body_set[ball.1].translation();
            canvas.draw(
                &ball.0,
                DrawParam::default().dest(Vec2::new(ball_body.x, ball_body.y)),
            );
        }

        for floor in &self.floors {
            canvas.draw(
                &floor.0,
                DrawParam::default().dest(Vec2::new(floor.2.x, floor.2.y)),
            );
        }

        self.draw_gizmos(&ctx, &mut canvas);

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
