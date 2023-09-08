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
}

struct State {
    circle: Mesh,
    ball: Ball 
}

impl State {
    fn new(ctx: &mut Context) -> State {
        State {
            circle: ggez::graphics::Mesh::new_circle(
                &ctx.gfx,
                DrawMode::Fill(FillOptions::DEFAULT),
                vec2(0., 0.),
                10.,
                10.,
                Color::BLUE
            ).unwrap(),
            ball: Ball {
                mass: 1.,
                pos: vec2(0., 0.),
                rope_len: 1.,
                rope_pivot: vec3(0.5, 0.5, 2.),
                velocity: vec3(0., 0., 0.)
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
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        canvas.draw(&self.circle, self.ball.pos * PIXELS_PER_METER);

        canvas.finish(&mut ctx.gfx)?;
        Ok(())
    }
}

fn main() {
    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("hello_ggez", "awesome_person")
        .default_conf(c)
        .build()
        .unwrap();

    let state = State::new(&mut ctx);

    event::run(ctx, event_loop, state);
}
