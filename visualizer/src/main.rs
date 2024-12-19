use ggez::event::{self, EventHandler};
use ggez::{graphics, Context, GameResult};
use rodio::source::{SamplesConverter, Source};
use rodio::{Decoder, OutputStream, OutputStreamHandle};
use std::fs::File;
use std::io::BufReader;

struct Visualizer {
    frames: usize,
    samples: SamplesConverter<Decoder<BufReader<std::fs::File>>, f32>,
    amplitudes: Vec<f32>,
}

impl Visualizer {
    fn update_amplitudes(&mut self) {
        let samples: Vec<f32> = self.samples.by_ref().take(512).collect();
        let avg_amp = samples.iter().map(|x| x.abs()).sum::<f32>() / samples.len() as f32;

        self.amplitudes.push(avg_amp * 500.0); // Scale amplitude for bars
        if self.amplitudes.len() > 30 {
            self.amplitudes.remove(0); // Keep only recent amplitudes
        }
    }
}

impl EventHandler for Visualizer {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.update_amplitudes();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Draw title
        let title_pos = ggez::glam::Vec2::new(20.0, 20.0);
        canvas.draw(
            graphics::Text::new("Music Visualizer").set_scale(32.),
            title_pos,
        );

        // Draw amplitude bars
        let bar_width = 20.0;
        let spacing = 5.0;
        let start_x = 50.0;
        let base_y = 400.0;

        for (i, &amp) in self.amplitudes.iter().enumerate() {
            let x = start_x + i as f32 * (bar_width + spacing);
            let height = amp.max(5.0); // Minimum height for visibility
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, base_y - height, bar_width, height),
                graphics::Color::new(0.0, 0.8, 1.0, 1.0),
            )?;
            canvas.draw(&rect, graphics::DrawParam::default());
        }

        canvas.finish(ctx)?;

        self.frames += 1;
        if self.frames % 100 == 0 {
            println!("FPS: {}", ctx.time.fps());
        }

        Ok(())
    }
}

pub fn main() -> GameResult {
    // Set up audio
    let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("music.mp3").unwrap());
    let source = Decoder::new(file).unwrap();

    // Play audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    // Set up visualizer
    let samples = source.convert_samples();
    let cb = ggez::ContextBuilder::new("music_visualizer", "Your Name");
    let (ctx, event_loop) = cb.build()?;
    let vis = Visualizer {
        frames: 0,
        samples,
        amplitudes: vec![],
    };

    event::run(ctx, event_loop, vis)
}
