mod par;

use std::path::Path;

use ggez::*;
use ggez::glam::*;
use ggez::graphics::*;

use image::{RgbImage, Rgb};

fn canvas_position(pos: Vec2, ctx: &mut Context, physics_ctx: &PhysicsContext) -> Vec2 {
    let center: Vec2 = ctx.gfx.size().into();
    let center = center / 2.;
    center + pos * physics_ctx.pixels_per_meter
}

fn world_position(pos: Vec2, ctx: &mut Context, physics_ctx: &PhysicsContext) -> Vec2 {
    let center: Vec2 = ctx.gfx.size().into();
    let center = center / 2.;
    (pos - center) / physics_ctx.pixels_per_meter
}

fn world_position2(pos: Vec2, center: Vec2, physics_ctx: &PhysicsContext) -> Vec2 {
    (pos - center) / physics_ctx.pixels_per_meter
}

fn angle3(x1: Vec3, x2: Vec3) -> f32 {
    (x1.dot(x2) * x1.length_recip() * x2.length_recip()).acos()
}

struct PhysicsContext {
    gravity: f32,
    pixels_per_meter: f32,
    magnet_coefficent: f32,
    time_precision: f32,
    speed: f32
}

impl PhysicsContext {
    fn new() -> PhysicsContext {
        PhysicsContext { 
            gravity: 10., 
            pixels_per_meter: 3000., 
            magnet_coefficent: 0.0002,
            time_precision: 0.0001,
            speed: 0.7
        }
    }
}

struct Ball {
    mass: f32,
    pos: Vec2,
    rope_len: f32,
    rope_pivot: Vec3,
    velocity: Vec3,
    air_friction: f32,
    magnets: Vec<Vec3>,
    last_positions: Vec<Vec2>,
}

impl Ball {
    fn ball_height(&self) -> f32 {
        let a = self.pos.distance(self.rope_pivot.xy());
        let c = self.rope_len;
        let b = self.rope_pivot.z - ((c - a) * (c + a)).sqrt();
        b
    }

    fn move_step(&mut self, physics_ctx: &PhysicsContext) {
        let ball_pos = vec3(self.pos.x, self.pos.y, self.ball_height());
        let gravity_force = vec3(0., 0., -1. * physics_ctx.gravity * self.mass);
        let friction_force = self.velocity.normalize_or_zero() * self.velocity.length_squared() * self.air_friction * -1.;

        let mut magnetic_force = vec3(0., 0., 0.);
        for magnet in self.magnets.iter() {
            let magnet_force = *magnet - ball_pos; 
            let magnitude = physics_ctx.magnet_coefficent / magnet_force.length_squared();
            let magnet_force = magnet_force.normalize() * magnitude;

            magnetic_force += magnet_force;
        }

        let force_vector = gravity_force + magnetic_force + friction_force;
        let rope_vector = self.rope_pivot - ball_pos;

        let force_projected = (force_vector.dot(rope_vector) / rope_vector.length_squared()) * rope_vector;
        let angle = angle3(force_projected, force_vector).to_degrees();
        let force_projected = if angle < 90. {
            force_projected * -1.
        } else {
            force_projected
        };

        let force = force_vector + force_projected;

        let a = force / self.mass;
        self.velocity += a * physics_ctx.time_precision;
        self.pos += self.velocity.xy() * physics_ctx.time_precision;
    }

    fn move_over_speed1(&mut self, time_delta: f32, physics_ctx: &PhysicsContext) {
        let times = (time_delta / physics_ctx.time_precision).floor() as u32;
        for _ in 0..times {
            self.move_step(physics_ctx);
        }
    }

    fn move_over_time(&mut self, time_delta: f32, physics_ctx: &PhysicsContext) {
        let times = (time_delta * physics_ctx.speed / physics_ctx.time_precision).floor() as u32;
        for _ in 0..times {
            self.move_step(physics_ctx);
        }
    }

    fn move_over_time_save_positions(&mut self, time_delta: f32, physics_ctx: &PhysicsContext) {
        let times = (time_delta * physics_ctx.speed / physics_ctx.time_precision).floor() as u32;
        let positions: Vec<_> = (0..times).map(|_| {
            self.move_step(physics_ctx);
            self.pos.clone()
        }).collect();

        self.last_positions = positions;
    }
}

struct Meshes {
    ball: Mesh,
    magnet: Mesh,
    trail: Mesh,
}

impl Meshes {
    fn new(ctx: &mut Context) -> Meshes {
        Meshes {
            ball: Mesh::new_circle(
                &ctx.gfx,
                DrawMode::Fill(FillOptions::DEFAULT),
                vec2(0., 0.),
                10.,
                0.01,
                Color::BLUE
            ).unwrap(),
            trail: Mesh::new_rectangle(
                &ctx.gfx,
                DrawMode::Fill(FillOptions::DEFAULT),
                Rect { x: 1., y: 1., w: 2., h: 2. },
                Color::RED
            ).unwrap(),
            magnet: Mesh::new_rectangle(
                &ctx.gfx,
                DrawMode::Fill(FillOptions::DEFAULT),
                Rect { x: 5., y: 5., w: 10., h: 10. },
                Color::BLACK
            ).unwrap(),
        }
    }
}

struct State {
    trail: ScreenImage,
    ball: Ball,
    meshes: Meshes,
    physics_ctx: PhysicsContext
}

impl State {
    fn new(pos: Vec2, ctx: &mut Context) -> State {
        State {
            trail: ScreenImage::new(
                &ctx.gfx, 
                None, 
                1., 
                1., 
                1
            ),
            ball: Ball {
                // r = 0.02
                mass: 0.264,
                pos,
                rope_len: 0.3,
                rope_pivot: vec3(0., 0., 0.33),
                velocity: vec3(0., 0., 0.),
                air_friction: 0.037,
                magnets: vec![
                    vec3((30.0 as f32).to_radians().cos(), (30.0 as f32).to_radians().sin(), 1.) * 0.04,
                    vec3((150.0 as f32).to_radians().cos(), (150.0 as f32).to_radians().sin(), 1.) * 0.04,
                    vec3((270.0 as f32).to_radians().cos(), (270.0 as f32).to_radians().sin(), 1.) * 0.04
                ],
                last_positions: vec![]
            },
            meshes: Meshes::new(ctx),
            physics_ctx: PhysicsContext::new()
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.ball.move_over_time_save_positions(ctx.time.delta().as_secs_f32(), &self.physics_ctx);
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if ctx.mouse.button_just_pressed(event::MouseButton::Left) {
            *self = State::new(world_position(ctx.mouse.position().into(), ctx, &self.physics_ctx), ctx);
        }
        self.update(ctx);

        // println!("velocity: {}", self.ball.velocity.xy().length());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> { 
        let mut trail_canvas = Canvas::from_screen_image(ctx, &mut self.trail, None);

        let mut last_pos = vec2(0., 0.);
        for pos in self.ball.last_positions.drain(0..).map(|x| canvas_position(x, ctx, &self.physics_ctx)) {
            trail_canvas.draw(&self.meshes.trail, pos);
            last_pos = pos;
        }

        trail_canvas.finish(&mut ctx.gfx)?;

        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
        self.trail.image(&mut ctx.gfx).draw(&mut canvas, DrawParam::new());
        for magnet in self.ball.magnets.iter() {
            canvas.draw(&self.meshes.magnet, canvas_position(magnet.xy(), ctx, &self.physics_ctx))
        }
        canvas.draw(&self.meshes.ball, last_pos);

        canvas.finish(&mut ctx.gfx)?;

        Ok(())
    }
}

// fn main() {
//     let (w, h) = (400, 400);
//     let center = vec2(w as f32 / 2., h as f32 / 2.);
//     let mut img = RgbImage::new(w, h);
//
//     let mut physics_ctx = PhysicsContext::new();
//     physics_ctx.time_precision = 0.01;
//     let mut ball = Ball {
//         // r = 0.02
//         mass: 0.264,
//         pos: vec2(0., 0.),
//         rope_len: 0.3,
//         rope_pivot: vec3(0., 0., 0.33),
//         velocity: vec3(0., 0., 0.),
//         air_friction: 0.037,
//         magnets: vec![
//             vec3((30.0 as f32).to_radians().cos(), (30.0 as f32).to_radians().sin(), 1.) * 0.04,
//             vec3((150.0 as f32).to_radians().cos(), (150.0 as f32).to_radians().sin(), 1.) * 0.04,
//             vec3((270.0 as f32).to_radians().cos(), (270.0 as f32).to_radians().sin(), 1.) * 0.04
//         ],
//         last_positions: vec![]
//     };
//
//     let color0 = Rgb([255, 0, 0]);
//     let color1 = Rgb([255, 255, 0]);
//     let color2 = Rgb([0, 0, 255]);
//
//     for x in 0..w {
//         for y in 0..h {
//             let pos = world_position2(vec2(x as f32, y as f32), center, &physics_ctx);
//             ball.pos = pos;
//             ball.velocity = vec3(0., 0., 0.);
//             ball.move_over_speed1(30., &physics_ctx);
//
//             let end_pos = ball.pos;
//
//             let mut closest_magnet = 0;
//             let mut min_distance = end_pos.distance(ball.magnets[0].xy()); 
//             for (i, magnet_pos) in ball.magnets.iter().enumerate().skip(1) {
//                 let d = end_pos.distance(magnet_pos.xy());
//                 if d < min_distance {
//                     closest_magnet = i;
//                     min_distance = d;
//                 }
//             }
//
//             let color = match closest_magnet {
//                 0 => color0,
//                 1 => color1,
//                 2 => color2,
//                 _ => panic!("weird index")
//             };
//
//             img.put_pixel(x, y, color)
//         }
//     }
//
//     img.save(Path::new("./test.png")).unwrap();
//
//     let c = conf::Conf::new();
//     let (mut ctx, event_loop) = ContextBuilder::new("magpen", "rubydusa")
//         .default_conf(c)
//         .build()
//         .unwrap();
//
//     // let image = Image::from_pixels(&mut ctx, &pixels, ImageFormat::Rgba8Uint, w, h);
//     // image.encode(&mut ctx, ImageEncodingFormat::Png, Path::new("./test.png")).unwrap();
//
//     let state = State::new(vec2(0., 0.), &mut ctx);
//     event::run(ctx, event_loop, state);
// }
fn main () {
    par::run();
}
