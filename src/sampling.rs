use crate::{Point2f, Vec2f, Vec3f, Float};
use std::f32;

pub fn concentric_sample_disk(u: Point2f) -> Point2f {
    // map sample from [0, 1] to [-1, 1]
    let u_offset = 2.0 * u - Vec2f::new(1.0, 1.0);
    if u_offset == Point2f::new(0.0, 0.0) {
        return Point2f::new(0.0, 0.0);
    }

    let (theta, r) = if u_offset.x.abs() > u_offset.y.abs() {
        (u_offset.x, f32::consts::FRAC_PI_4 * (u_offset.y / u_offset.x))
    } else {
        (u_offset.y, f32::consts::FRAC_PI_2 - f32::consts::FRAC_PI_4 * (u_offset.x / u_offset.y))
    };

    r * Point2f::new(theta.cos(), theta.sin())
}

pub fn cosine_sample_hemisphere(u: Point2f) -> Vec3f {
    let d = concentric_sample_disk(u);
    let z = Float::sqrt(Float::max(0.0, 1.0 - d.x * d.x - d.y * d.y));
    Vec3f::new(d.x, d.y, z)
}