use std::path::Path;

use ggez::glam::*;
use image::{Rgb, RgbImage};
use ndarray::{prelude::*, concatenate};

struct Magnet {
    position: DVec3,
    color: Rgb<u8>
}

struct Context {
    gravity: f64,
    mass: f64,
    rope_length: f64,
    rope_pivot: DVec3,
    air_resistence_coefficent: f64,
    magnet_coefficent: f64,
    time_step: f64,
    magnets: Vec<Magnet>,
    meters_per_unit: f64
}

struct State<'a> {
    size: (usize, usize),
    position: Array3<f64>,
    velocity: Array3<f64>,
    ctx: &'a Context
}

impl<'a> State<'a> {
    fn new(x: usize, y: usize, ctx: &'a Context) -> State<'a> {
        let center = dvec2((x / 2) as f64, (y / 2) as f64);
        let position = Array3::from_shape_fn((x, y, 3), |(i, j, z)| {
            let pos = (dvec2(i as f64, j as f64) - center) * ctx.meters_per_unit;
            if z == 0 {
                pos.x
            } else if z == 1 {
                pos.y
            } else {
                let a = pos.distance(ctx.rope_pivot.xy());
                let c = ctx.rope_length;
                let b = ctx.rope_pivot.z - ((c - a) * (c + a)).sqrt();

                b
            }
        });

        let velocity = Array3::zeros((x, y, 3));
        State {
            size: (x, y),
            position,
            velocity,
            ctx
        }
    }

    fn run_step(&mut self) {
        take_step(&mut self.position, &mut self.velocity, &self.ctx)
    }

    fn run(&mut self, seconds: f64) {
        let steps = (seconds / self.ctx.time_step).floor() as u32;
        for _ in 0..steps { 
            self.run_step();
        };
    }

    fn image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.size.0 as u32, self.size.1 as u32);
        let mut index_arr = Array2::zeros((self.size.0, self.size.1));
        let mut min_arr = Array2::from_elem((self.size.0, self.size.1), f64::MAX);

        for (i, magnet) in self.ctx.magnets.iter().enumerate() {
            let magnet_array = vector3_matrix(self.size.0, self.size.1, magnet.position);
            let distance_array = magnet_array - &self.position;
            let distance_array = &distance_array * &distance_array;
            let distance_array = distance_array.sum_axis(Axis(2));

            let mask = (&distance_array - &min_arr).mapv(|x| ((x < 0.) as u32) as f64);
            let mask_inverse = 1. - &mask;
            let magnet_index_arr = &mask * (i as f64);

            min_arr = distance_array * &mask + min_arr * &mask_inverse;
            index_arr = magnet_index_arr * mask + index_arr * mask_inverse;
        }

        for ((x, y), i) in index_arr.indexed_iter() {
            let magnet = i.round() as usize;
            let color = self.ctx.magnets[magnet].color;

            img.put_pixel(x as u32, y as u32, color);
        }

        img
    }
}

fn vector3_matrix(w: usize, h: usize, v: DVec3) -> Array3<f64> {
    concatenate![
        Axis(2),
        Array::from_elem((w, h, 1), v.x),
        concatenate![
            Axis(2),
            Array::from_elem((w, h, 1), v.y),
            Array::from_elem((w, h, 1), v.z)
        ]
    ]
}

fn as_uniform_vector(array: &Array2<f64>, len: usize) -> Array3<f64> {
    let mut expanded_array = array.clone().insert_axis(Axis(2));

    for _ in 0..(len - 1) {
        expanded_array.push(Axis(2), array.into()).unwrap();
    }

    expanded_array
}

fn dot(a: &Array3<f64>, b: &Array3<f64>) -> Array2<f64> {
    (a * b).sum_axis(Axis(2))
}

fn vector_lengths(array: &Array3<f64>) -> Array3<f64> {
    let squared = array * array;
    let length_squared = squared.sum_axis(Axis(2));
    let length = length_squared.mapv(f64::sqrt);
    as_uniform_vector(&length, array.len_of(Axis(2)))
}

fn vector_squared_lengths(array: &Array3<f64>) -> Array3<f64> {
    let squared = array * array;
    let length_squared = squared.sum_axis(Axis(2));
    as_uniform_vector(&length_squared, array.len_of(Axis(2)))
}

fn normalize(array: &mut Array3<f64>) {
    let lengths = vector_lengths(&*array);
    *array = &*array / lengths;
    array.mapv_inplace(|x| if x.is_nan() { 0. } else { x });
}

fn take_step(
    position: &mut Array3<f64>, 
    velocity: &mut Array3<f64>, 
    ctx: &Context
) {
    let shape = position.shape();

    let mut gravity_force = Array::<f64, _>::zeros((
        shape[0],
        shape[1],
        2
    ));

    gravity_force.push(Axis(2), (&Array::from_elem((shape[0], shape[1]), ctx.mass * ctx.gravity)).into()).unwrap();
    let mut air_resistence_force = velocity.clone();
    normalize(&mut air_resistence_force);
    air_resistence_force = &air_resistence_force * vector_squared_lengths(velocity) * ctx.air_resistence_coefficent * -1.;

    let mut total_magnetic_force = Array::<f64, _>::zeros([shape[0], shape[1], shape[2]]);
    for magnet in ctx.magnets.iter() {
        let magnet_position_array = vector3_matrix(shape[0], shape[1], magnet.position);

        let mut magnetic_force = &magnet_position_array - &*position;
        let magnitude = ctx.magnet_coefficent / vector_squared_lengths(&magnetic_force);
        normalize(&mut magnetic_force);
        magnetic_force = &magnetic_force * magnitude;
        total_magnetic_force = &total_magnetic_force + magnetic_force;
    }

    let force_vectors = gravity_force + air_resistence_force + total_magnetic_force;
    let rope_pivot_array = vector3_matrix(shape[0], shape[1], ctx.rope_pivot);
    let rope_vectors = &rope_pivot_array - &*position;
    // forces projected onto the normal of the movement plane
    let forces_projected = 
        as_uniform_vector(&dot(&force_vectors, &rope_vectors), 3) / 
        vector_squared_lengths(&rope_vectors) 
        * -1.
        * rope_vectors;

    let final_force = force_vectors + forces_projected;
    let a = final_force / ctx.mass;
    *velocity = &*velocity + a * ctx.time_step;
    *position = &*position + &*velocity * ctx.time_step;

    // fix position y
    let mut position2d = position.clone();
    position2d.remove_index(Axis(2), 2);
    let mut rope_pivot_array2d = rope_pivot_array.clone();
    rope_pivot_array2d.remove_index(Axis(2), 2);

    let a = position2d - rope_pivot_array2d;
    let a = a.sum_axis(Axis(2));
    let c = ctx.rope_length;
    let b = ctx.rope_pivot.z - ((&a + c) * (a - c)).mapv(f64::sqrt);

    position.remove_index(Axis(2), 2);
    *position = concatenate![Axis(2), position.view(), b.insert_axis(Axis(2))];
}

pub fn run() {
    let ctx = Context {
        gravity: 10.,
        mass: 1.,
        rope_length: 1.,
        rope_pivot: dvec3(0., 0., 1.04),
        air_resistence_coefficent: 0.037,
        magnet_coefficent: 0.0002,
        time_step: 0.0001,
        magnets: vec![
            Magnet { 
                position: dvec3((30.0 as f64).to_radians().cos(), (30.0 as f64).to_radians().sin(), 1.) * 0.04,
                color: Rgb([255, 0, 0])
            },
            Magnet { 
                position: dvec3((150.0 as f64).to_radians().cos(), (150.0 as f64).to_radians().sin(), 1.) * 0.04,
                color: Rgb([0, 255, 0])
            },
            Magnet { 
                position: dvec3((270.0 as f64).to_radians().cos(), (270.0 as f64).to_radians().sin(), 1.) * 0.04,
                color: Rgb([255, 255, 0])
            }
        ],
        meters_per_unit: 0.001
    };
    
    let mut state = State::new(200, 200, &ctx);
    state.run(0.01);
    state.image().save(Path::new("./numpy.png")).unwrap();
}
