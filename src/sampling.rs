use crate::{Point2f, Vec2f, Vec3f, Float, Point3f};
use std::f32;
use rand::Rng;

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

pub fn rejection_sample_shere(rng: &mut impl Rng, radius: Float) -> Point3f {
    loop {
        let x = rng.gen_range(-radius, radius);
        let y = rng.gen_range(-radius, radius);
        let z = rng.gen_range(-radius, radius);
        let d = x * x + y * y + z * z;
        if d < radius * radius { break Point3f::new(x, y, z) }
    }
}

pub fn uniform_sample_sphere(u: Point2f) -> Vec3f {
    let z = 1.0 - 2.0 * u[0];
    let r = 0.0.max(1.0 - z * z).sqrt();
    let phi = 2.0 * std::f32::consts::PI * u[1];
    Vec3f::new(r * phi.cos(), r * phi.sin(), z)
}

pub const fn uniform_sphere_pdf() -> Float {
    std::f32::consts::FRAC_1_PI * 4.0
}

pub fn uniform_sample_triangle(u: Point2f) -> Point2f {
    let su0 = u[0].sqrt();
    Point2f::new(1 - su0, u[1] * su0)
}