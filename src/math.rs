use crate::EFloat;
use crate::Float;
use crate::err_float::MACHINE_EPSILON;

pub const INFINITY: Float = std::f32::INFINITY;

pub fn lerp(t: Float, v1: Float, v2: Float) -> Float {
    (1.0 - t) * v1 + t * v2
}

pub fn quadratic(a: EFloat, b: EFloat, c: EFloat) -> Option<(EFloat, EFloat)> {
    let discrim: f64 = b.v as f64 * b.v as f64 - (4.0 * a.v as f64 * c.v as f64);
    if discrim < 0.0 { return None; }

    let root_discrim = discrim.sqrt();
    let root_discrim = EFloat::with_err(root_discrim as Float, MACHINE_EPSILON * root_discrim as Float);

    let q: EFloat = if b.v < 0.0 {
        -0.5 * (b - root_discrim)
    } else {
        -0.5 * (b + root_discrim)
    };

    let t0 = q / a;
    let t1 = c / q;

    if t0.v > t1.v { Some((t1, t0)) } else { Some((t0, t1)) }
}
