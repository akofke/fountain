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
    let z: Float = 1.0 - 2.0 * u[0];
    let r: Float = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * std::f32::consts::PI * u[1];
    Vec3f::new(r * phi.cos(), r * phi.sin(), z)
}

pub const fn uniform_sphere_pdf() -> Float {
    std::f32::consts::FRAC_1_PI * 4.0
}

pub fn uniform_sample_triangle(u: Point2f) -> Point2f {
    let su0 = u[0].sqrt();
    Point2f::new(1.0 - su0, u[1] * su0)
}

pub fn power_heuristic(nf: u32, f_pdf: Float, ng: u32, g_pdf: Float) -> Float {
    let f = nf as Float * f_pdf;
    let g = ng  as Float * g_pdf;
    (f * f) / (f * f + g * g)
}

pub struct Distribution1D {
    func: Vec<Float>,
    cdf: Vec<Float>,
    func_integral: Float,
}

pub fn find_interval<F: Fn(usize) -> bool>(size: usize, key: F) -> usize {

}

impl Distribution1D {
    pub fn new(func: Vec<Float>) -> Self {
        let n = func.len();
        let mut cdf = vec![0.0; n + 1];

        // Compute the integral of the step function at x_i
        // FIXME: could try with iterators and scan
        for i in 1..(n + 1) {
            cdf[i] = cdf[i - 1] + (func[i - 1] / n as Float);
        }

        // Transform step function integral into cdf
        let func_integral = cdf[n];
        if func_integral == 0.0 {
            cdf.iter_mut().enumerate().for_each(|(i, x)| *x = i as Float / n as Float);
        } else {
            cdf.iter_mut().for_each(|x| *x /= func_integral);
        }

        Self {
            func,
            cdf,
            func_integral,
        }
    }

    pub fn func(&self) -> &[Float] {
        &self.func
    }

    /// Uses the given random variable `u` to sample from the distribution.
    /// Returns a tuple of `(x, p(x), idx)` containing the sampled `x in [0, 1)`,
    /// the value of the PDF `p(x)`, and the index into the array of function values where
    /// `cdf[n] <= u < cdf[n+1]`.
    pub fn sample_continuous(&self, u: Float) -> (Float, Float, usize) {
        unimplemented!()
    }
}