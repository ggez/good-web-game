//! An example of how to use a `MeshBatch`.

extern crate good_web_game as ggez;

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::miniquad;
use ggez::timer;
use ggez::{Context, GameResult};
use glam::*;
use oorandom::Rand32;
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

struct MainState {
    mesh_batch: graphics::MeshBatch,
}

impl MainState {
    fn new(ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult<MainState> {
        let mut rng = Rand32::new(12345);
        let mesh = graphics::MeshBuilder::new()
            .circle(
                graphics::DrawMode::stroke(4.0),
                Vec2::new(0.0, 0.0),
                8.0,
                1.0,
                (0, 0, 255).into(),
            )?
            .line(
                &[Vec2::new(0.0, 0.0), Vec2::new(8.0, 0.0)],
                2.0,
                (255, 255, 0).into(),
            )?
            .build(ctx, quad_ctx)?;

        let mut mesh_batch = graphics::MeshBatch::new(mesh)?;

        // Generate enough instances to fill the entire screen
        let items_x = (graphics::drawable_size(quad_ctx).0 / 16.0) as u32;
        let items_y = (graphics::drawable_size(quad_ctx).1 / 16.0) as u32;
        for x in 1..items_x {
            for y in 1..items_y {
                let x = x as f32;
                let y = y as f32;

                let p = graphics::DrawParam::new()
                    .dest(Vec2::new(x * 16.0, y * 16.0))
                    .rotation(rng.rand_float() * TWO_PI);

                mesh_batch.add(p);
            }
        }

        // Randomly shuffle generated instances.
        // We will update the first 50 of them later.
        mesh_batch.get_instance_params_mut();
        //.shuffle(&mut thread_rng());

        let s = MainState { mesh_batch };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    #[allow(clippy::needless_range_loop)]
    fn update(
        &mut self,
        ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
    ) -> GameResult {
        if timer::ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::delta(ctx));
            println!("Average FPS: {}", timer::fps(ctx));
        }

        // Update first 50 instances in the mesh batch
        let delta_time = (timer::duration_to_f64(timer::delta(ctx)) * 1000.0) as f32;
        let instances = self.mesh_batch.get_instance_params_mut();
        for i in 0..50 {
            if let graphics::Transform::Values {
                ref mut rotation, ..
            } = instances[i].trans
            {
                if (i % 2) == 0 {
                    *rotation += 0.001 * TWO_PI * delta_time;
                    if *rotation > TWO_PI {
                        *rotation -= TWO_PI;
                    }
                } else {
                    *rotation -= 0.001 * TWO_PI * delta_time;
                    if *rotation < 0.0 {
                        *rotation += TWO_PI;
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult {
        graphics::clear(ctx, quad_ctx, Color::BLACK);

        // Flush the first 50 instances in the batch to make our changes visible
        // to the graphics card.
        self.mesh_batch
            .flush_range(ctx, quad_ctx, graphics::MeshIdx(0), 50)?;

        // Draw the batch
        let param = graphics::DrawParam::default();
        // Uncomment this line to see the effect of an offset on a MeshBatch
        // This is somewhat expensive though, as it results in the MeshBatch
        // having to calculate its drawn dimensions every frame, to be able
        // to apply the offset based on that.
        //let param = param.offset([0.5, 0.5]);
        self.mesh_batch.draw(ctx, quad_ctx, param)?;

        graphics::present(ctx, quad_ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    ggez::start(ggez::conf::Conf::default(), |mut context, quad_ctx| {
        Box::new(MainState::new(&mut context, quad_ctx).unwrap())
    })
}
