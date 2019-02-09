use rand::prelude::*;
use nalgebra::Vector2;
use crate::Vec3;

pub fn random_in_unit_sphere() -> Vec3 {
    loop {
        let p = 2.0 * Vec3::new(random(), random(), random()) - Vec3::repeat(1.0);
        if p.norm_squared() < 1.0 { break p }
    }
}

pub fn random_in_unit_disk() -> Vector2<f32> {
    loop {
        let p: Vector2<f32> = 2.0 * Vector2::<f32>::new(random(), random()) - Vector2::repeat(1.0f32);
        if p.norm_squared() < 1.0 { break p }
    }
}
