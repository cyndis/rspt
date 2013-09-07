use std::rand;
use std::rand::RngUtil;
use nalgebra::vec::*;

pub fn random_real() -> float {
    rand::task_rng().gen_uint_range(0,1000) as float / 999.0
}

pub fn random_vec() -> Vec3<float> {
    let th = random_real() * 2.0 * 3.14;
    let z = -1.0 + 2.0 * random_real();
    let t = (1.0 - z * z).sqrt();

    Vec3::new(t * th.cos(), t * th.sin(), z)
}
