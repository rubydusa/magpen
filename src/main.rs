use ggez::*;
use ggez::glam::*;
use ggez::graphics::*;

fn canvas_position(pos: Vec2, ctx: &mut Context, physics_ctx: &PhysicsContext) -> Vec2 {
    let center: Vec2 = ctx.gfx.size().into();
    let center = center / 2.;
    center + pos * physics_ctx.pixels_per_meter
}

fn angle3(x1: Vec3, x2: Vec3) -> f32 {
    (x1.dot(x2) * x1.length_recip() * x2.length_recip()).acos()
}

struct PhysicsContext {
    gravity: f32,
    pixels_per_meter: f32,
    magnet_coefficent: f32,
    time_precision: f32
}

impl PhysicsContext {
    fn new() -> PhysicsContext {
        PhysicsContext { 
            gravity: 10., 
            pixels_per_meter: 100., 
            magnet_coefficent: 0.1, 
            time_precision: 0.001 
        }
    }
}

struct Ball {
    mass: f32,
    pos: Vec2,
    rope_len: f32,
    rope_pivot: Vec3,
    velocity: Vec3,
    magnets: Vec<Vec2>,
}

impl Ball {
    fn ball_height(&self) -> f32 {
        let a = self.pos.distance(self.rope_pivot.xy());
        let c = self.rope_len;
        let b = self.rope_pivot.z - ((c - a) * (c + a)).sqrt();
        b
    }

    fn move_self(&mut self, time_delta: f32, physics_ctx: &PhysicsContext) {
        let times = (time_delta / physics_ctx.time_precision).floor() as u32;
        for _ in 0..times {
            let ball_pos = vec3(self.pos.x, self.pos.y, self.ball_height());
            let gravity_force = vec3(0., 0., -1. * physics_ctx.gravity * self.mass);

            let mut magnetic_force = vec3(0., 0., 0.);
            for magnet in self.magnets.iter() {
               let magnet_force = vec3(magnet.x, magnet.y, 0.) - ball_pos; 
               let magnitude = physics_ctx.magnet_coefficent / magnet_force.length_squared();
               let magnet_force = magnet_force.normalize() * magnitude;

               magnetic_force += magnet_force;
            }

            let force_vector = gravity_force + magnetic_force;
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
    fn new(ctx: &mut Context) -> State {
        State {
            trail: ScreenImage::new(
                &ctx.gfx, 
                None, 
                1., 
                1., 
                1
            ),
            ball: Ball {
                mass: 0.1,
                pos: vec2(1., 1.),
                rope_len: 10.,
                rope_pivot: vec3(0., 0., 10.01),
                velocity: vec3(0., 0., 0.),
                magnets: vec![
                    vec2((30.0 as f32).to_radians().cos(), (30.0 as f32).to_radians().sin()),
                    vec2((150.0 as f32).to_radians().cos(), (150.0 as f32).to_radians().sin()),
                    vec2((270.0 as f32).to_radians().cos(), (270.0 as f32).to_radians().sin())
                ]
            },
            meshes: Meshes::new(ctx),
            physics_ctx: PhysicsContext::new()
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.ball.move_self(ctx.time.delta().as_secs_f32(), &self.physics_ctx);
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> { 
        let pos = canvas_position(self.ball.pos, ctx, &self.physics_ctx);

        let mut trail_canvas = Canvas::from_screen_image(ctx, &mut self.trail, None);
        trail_canvas.draw(&self.meshes.trail, pos);
        trail_canvas.finish(&mut ctx.gfx)?;

        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
        self.trail.image(&mut ctx.gfx).draw(&mut canvas, DrawParam::new());
        for magnet in self.ball.magnets.iter() {
            canvas.draw(&self.meshes.magnet, canvas_position(*magnet, ctx, &self.physics_ctx))
        }
        canvas.draw(&self.meshes.ball, pos);

        canvas.finish(&mut ctx.gfx)?;

        Ok(())
    }
}

fn main() {
    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("magpen", "rubydusa")
        .default_conf(c)
        .build()
        .unwrap();

    let state = State::new(&mut ctx);

    event::run(ctx, event_loop, state);
}
