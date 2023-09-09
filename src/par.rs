use ggez::glam::*;
use ndarray::{prelude::*, concatenate};

struct Magnet {
    position: DVec3
}

struct Context {
    gravity: f64,
    mass: f64,
    rope_length: f64,
    pivot_position: DVec3,
    air_resistence_coefficent: f64,
    magnet_coefficent: f64,
    time_step: f64,
    meters_per_unit: f64,
    magnets: Vec<Magnet>,
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

    for _ in 0..len {
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
    as_uniform_vector(&length, array.len_of(Axis(2)) - 1)
}

fn vector_squared_lengths(array: &Array3<f64>) -> Array3<f64> {
    let squared = array * array;
    let length_squared = squared.sum_axis(Axis(2));
    as_uniform_vector(&length_squared, array.len_of(Axis(2)) - 1)
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
    let rope_pivot_array = vector3_matrix(shape[0], shape[1], ctx.pivot_position);
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
    let c = ctx.rope_length;
    let b = ctx.pivot_position.z - ((&a + c) * (a - c)).mapv(f64::sqrt);

    position.remove_index(Axis(2), 2);
    *position = concatenate![Axis(2), position.view(), b];
}

pub fn run() {
    let mut ones = Array::<f64, _>::zeros((2, 2, 2));
    normalize(&mut ones);
    println!("after: {}", ones);
}
