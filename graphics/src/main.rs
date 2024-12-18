use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Color, Mesh, PxScale, Rect, TextFragment},
    mint, GameResult,
};
use rand::{rngs::ThreadRng, thread_rng, Rng};

struct Object {
    id: i32,
    width: f32,
    height: f32,
    pos_x: f32,
    pos_y: f32,
    move_x: f32,
    move_y: f32,
    dir_x: f32,
    dir_y: f32,
    color: Color,
    rng: ThreadRng,
    movement_speed: f32,
}

impl Object {
    fn render(&self, ctx: &ggez::Context) -> GameResult<Mesh> {
        Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, self.width, self.height),
            self.color,
        )
    }

    fn random_move(&mut self) {
        if self.move_x == 0.0 && self.move_y == 0.0 {
            self.move_x = self.rng.gen_range(0.0..200.0);
            self.move_y = self.rng.gen_range(0.0..200.0);
            self.dir_x = if self.rng.gen() { -1.0 } else { 1.0 };
            self.dir_y = if self.rng.gen() { -1.0 } else { 1.0 }
        }
    }

    fn walk(&mut self, world_bounder_x: f32, world_bounder_y: f32) {
        if self.move_x != 0.0 {
            let mut new_x = self.pos_x + (self.movement_speed * self.dir_x);
            self.move_x -= self.movement_speed;

            if self.move_x < 0.0 {
                self.move_x = 0.0
            }

            if new_x < 0.0 {
                new_x = 0.0;
            }

            if new_x > (world_bounder_x - self.width) {
                new_x = world_bounder_x - self.width;
            }

            self.pos_x = new_x;
        }
        if self.move_y != 0.0 {
            let mut new_y = self.pos_y + (self.movement_speed * self.dir_y);
            self.move_y -= self.movement_speed;

            if self.move_y < 0.0 {
                self.move_y = 0.0
            }

            if new_y < 0.0 {
                new_y = 0.0;
            }
            if new_y > (world_bounder_y - self.height) {
                new_y = world_bounder_y - self.height;
            }

            self.pos_y = new_y;
        }
    }

    fn _random_movement(&mut self, world_bounder_x: f32, world_bounder_y: f32) {
        let rand_x = self
            .rng
            .gen_range(-self.movement_speed..self.movement_speed);
        let rand_y = self
            .rng
            .gen_range(-self.movement_speed..self.movement_speed);
        let mut new_x: f32;
        let mut new_y: f32;

        new_x = self.pos_x + rand_x;
        new_y = self.pos_y + rand_y;

        if self.pos_x < 0.0 {
            new_x = 0.0
        }

        if self.pos_y < 0.0 {
            new_y = 0.0;
        }

        if self.pos_x > (world_bounder_x - self.width) {
            new_x = world_bounder_x - self.width
        }
        if self.pos_y > (world_bounder_y - self.height) {
            new_y = world_bounder_y - self.height
        }
        self.pos_x = new_x;
        self.pos_y = new_y;
    }
}

struct MainState {
    objects: Vec<Object>,
    canvas_width: f32,
    canvas_height: f32,
    rng: ThreadRng,
}

impl MainState {
    fn new(width: f32, height: f32, objects_count: i32) -> GameResult<MainState> {
        let mut rng = thread_rng();
        let mut objects = vec![];

        for id in 0..objects_count {
            let obj_width = rng.gen_range(30.0..60.0);
            let obj_height = rng.gen_range(30.0..60.0);
            let r = rng.gen_range(0..250);
            let g = rng.gen_range(0..250);
            let b = rng.gen_range(0..250);
            let obj_color = Color::from_rgb(r, g, b);

            objects.push(Object {
                id: id,
                width: obj_width,
                height: obj_height,
                pos_x: (width / 2.0) - (obj_width / 2.0),
                pos_y: (height / 2.0) - (obj_height / 2.0),
                color: obj_color,
                rng: thread_rng(),
                movement_speed: rng.gen_range(1.0..10.0),
                move_x: 0.0,
                move_y: 0.0,
                dir_x: 0.0,
                dir_y: 0.0,
            });
        }

        let state = MainState {
            objects,
            canvas_width: width,
            canvas_height: height,
            rng: thread_rng(),
        };

        Ok(state)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let (width, height) = ctx.gfx.drawable_size();

        self.canvas_height = height;
        self.canvas_width = width;

        if ctx.time.check_update_time(30) {
            for obj in self.objects.iter_mut() {
                if self.rng.gen() && self.rng.gen() {
                    obj.random_move();
                }
                obj.walk(self.canvas_width, self.canvas_height);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::CYAN);

        for obj in self.objects.iter_mut() {
            let mesh = obj.render(ctx)?;
            canvas.draw(
                &mesh,
                graphics::DrawParam::new().dest(Vec2::new(obj.pos_x, obj.pos_y)),
                //graphics::DrawParam::new().dest(Vec2::new(0.0, self.canvas_height - obj.height)),
            )
        }

        canvas.draw(
            &graphics::Text::new(
                TextFragment::new(format!("FPS: {}", &ctx.time.fps()))
                    .color(Color::BLACK)
                    .scale(PxScale::from(50.0)),
            ),
            graphics::DrawParam::new().dest(Vec2::new(0.0, 0.0)),
        );

        canvas.finish(ctx)
    }
}

fn main() {
    let width = 1200.0;
    let height = 600.0;
    let context_builder = ggez::ContextBuilder::new("Playing", "Rui").window_mode(
        WindowMode::default()
            .dimensions(width, height)
            //.borderless(true)
            .resizable(true),
    );
    let (ctx, event_loop) = context_builder.build().unwrap();
    let objects_count = 100;
    let state = MainState::new(width, height, objects_count).unwrap();
    event::run(ctx, event_loop, state)
}
