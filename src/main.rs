use ggez::*;
use ggez::glam::*;
use ggez::graphics::*;

const GRAVITY: f32 = 10.;
const PIXELS_PER_METER: f32 = 100.;

struct Ball {
    mass: f32,
    pos: Vec2,
    rope_len: f32,
    rope_pivot: Vec3,
    velocity: Vec3
}

impl Ball {
    fn tension(&self) -> f32 {
        GRAVITY * self.mass
    }

    fn ball_height(&self) -> f32 {
        let a = self.pos.distance(self.rope_pivot.xy());
        let c = self.rope_len;
        (c - a) * (c + a)
    }

    fn move_self(&mut self, time_delta: f32) {
        let ball_pos = vec3(self.pos.x, self.pos.y, self.ball_height());
        let rope_vec = self.rope_pivot - ball_pos;
        let rope_force = rope_vec.normalize() * self.tension();
        let gravity_force = vec3(0., 0., -1. * GRAVITY * self.mass);
        let force = rope_force + gravity_force;
        let a = force / self.mass;
        self.velocity += a * time_delta;

        self.pos += self.velocity.xy() * time_delta;
    }

    fn canvas_position(&self, ctx: &mut Context) -> Vec2 {
        let center: Vec2 = ctx.gfx.size().into();
        let center = center / 2.;
        center + self.pos * PIXELS_PER_METER
    }
}

struct State {
    bg: ScreenImage,
    circle: Mesh,
    trail: Mesh,
    ball: Ball 
}

impl State {
    fn new(ctx: &mut Context) -> State {
        State {
            bg: ScreenImage::new(
                &ctx.gfx, 
                None, 
                1., 
                1., 
                1
            ),
            circle: Mesh::new_circle(
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
                Rect { x: 0., y: 0., w: 3., h: 3. },
                Color::BLUE
            ).unwrap(),
            ball: Ball {
                mass: 1.,
                pos: vec2(0.5, 0.5),
                rope_len: 1.,
                rope_pivot: vec3(0., 0., 2.),
                velocity: vec3(1., 0., 0.)
            }
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.ball.move_self(ctx.time.delta().as_secs_f32());
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> { 
        let pos = self.ball.canvas_position(ctx);
        let mut canvas = Canvas::from_screen_image(ctx, &mut self.bg, None);
        canvas.draw(&self.trail, pos);
        canvas.finish(&mut ctx.gfx)?;

        let mut canvas2 = Canvas::from_frame(ctx, Color::WHITE);
        self.bg.image(&mut ctx.gfx).draw(&mut canvas2, DrawParam::new());
        canvas2.draw(&self.circle, pos);
        canvas2.finish(&mut ctx.gfx)?;

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
