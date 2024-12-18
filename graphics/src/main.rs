use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Color, DrawParam, Mesh, MeshBuilder, PxScale, Rect, TextFragment},
    GameResult,
};
use rand::{rngs::ThreadRng, thread_rng, Rng};

const SCALE: f32 = 1.0;

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
    movement_speed: f32,
}

impl Object {
    fn build(&self) -> DrawParam {
        DrawParam::new()
            .dest(Vec2::new(self.pos_x, self.pos_y))
            .scale(Vec2::new(self.width, self.height))
            .color(self.color)
    }

    fn random_move(&mut self, rng: &mut ThreadRng) {
        if self.move_x == 0.0 && self.move_y == 0.0 {
            self.move_x = rng.gen_range(0.0..200.0);
            self.move_y = rng.gen_range(0.0..200.0);
            self.dir_x = if rng.gen() { -1.0 } else { 1.0 };
            self.dir_y = if rng.gen() { -1.0 } else { 1.0 };
        }
    }

    fn walk(&mut self, world_bounder_x: f32, world_bounder_y: f32, delta_time: f32) {
        let movement = self.movement_speed * delta_time;

        if self.move_x > 0.0 {
            let new_x =
                (self.pos_x + (movement * self.dir_x)).clamp(0.0, world_bounder_x - self.width);
            self.move_x = (self.move_x - movement).max(0.0);
            self.pos_x = new_x;
        }
        if self.move_y > 0.0 {
            let new_y =
                (self.pos_y + (movement * self.dir_y)).clamp(0.0, world_bounder_y - self.height);
            self.move_y = (self.move_y - movement).max(0.0);
            self.pos_y = new_y;
        }
    }
}

struct MainState {
    objects: Vec<Object>,
    mesh_batch: graphics::InstanceArray,
    mesh: graphics::Mesh,
    canvas_width: f32,
    canvas_height: f32,
    rng: ThreadRng,
}

impl MainState {
    fn new(
        ctx: &ggez::Context,
        width: f32,
        height: f32,
        objects_count: i32,
    ) -> GameResult<MainState> {
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
                width: obj_width / SCALE,
                height: obj_height / SCALE,
                pos_x: (width / 2.0) - (obj_width / 2.0),
                pos_y: (height / 2.0) - (obj_height / 2.0),
                color: obj_color,
                movement_speed: rng.gen_range(100.0..200.0),
                move_x: 0.0,
                move_y: 0.0,
                dir_x: 0.0,
                dir_y: 0.0,
            });
        }

        let mesh = graphics::Mesh::from_data(
            ctx,
            graphics::MeshBuilder::new()
                .rectangle(
                    graphics::DrawMode::fill(),
                    Rect::new(0.0, 0.0, SCALE, SCALE),
                    Color::WHITE,
                )?
                .build(),
        );

        let mut state = MainState {
            objects,
            canvas_width: width,
            canvas_height: height,
            rng: thread_rng(),
            mesh_batch: graphics::InstanceArray::new(ctx, None),
            mesh,
        };

        state.mesh_batch.resize(ctx, objects_count as usize);

        Ok(state)
    }
}

impl EventHandler for MainState {
    fn resize_event(
        &mut self,
        _ctx: &mut ggez::Context,
        width: f32,
        height: f32,
    ) -> Result<(), ggez::GameError> {
        self.canvas_height = height;
        self.canvas_width = width;

        Ok(())
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        for obj in self.objects.iter_mut() {
            if self.rng.gen() && self.rng.gen() {
                obj.random_move(&mut self.rng);
            }
            obj.walk(
                self.canvas_width,
                self.canvas_height,
                ctx.time.delta().as_secs_f32(),
            );
        }

        let instance_params = self.objects.iter().map(|obj| obj.build());

        self.mesh_batch.set(instance_params);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::CYAN);

        canvas.draw_instanced_mesh(self.mesh.clone(), &self.mesh_batch, DrawParam::default());

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

    let objects_count = 50000;
    let state = MainState::new(&ctx, width, height, objects_count).unwrap();
    event::run(ctx, event_loop, state)
}
