use ndarray::prelude::*;

struct Context {
    gravity: f64,
    mass: f64,
    friction_coefficent: f64,
    time_step: f64,
    meters_per_unit: f64
}

fn normalize(array: &mut Array3<f64>) {
    let squared = &*array * &*array;
    let length_squared = squared.sum_axis(Axis(2));
    let length = length_squared.mapv(f64::sqrt);
    let mut length_expanded = length.clone().insert_axis(Axis(2));

    for _ in 0..(array.len_of(Axis(2)) - 1) {
        length_expanded.push(Axis(2), (&length).into()).unwrap();
    }

    *array = length_expanded * &*array;
}

fn take_step(
    position: &mut Array3<f64>, 
    velocity: &mut Array3<f64>, 
    ctx: &Context
) {
    let gravity_force = ctx.mass * ctx.gravity;

}

pub fn run() {
    let mut ones = Array::<f64, _>::ones((2, 2, 2));
    normalize(&mut ones);
    println!("after: {}", ones);
}
