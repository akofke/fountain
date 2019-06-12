use crate::Float;

pub const MACHINE_EPSILON: f32 = std::f32::EPSILON * 0.5;

pub const fn gamma(n: i32) -> Float {
    let n = n as Float;
    (n * MACHINE_EPSILON) / (1.0 - n * MACHINE_EPSILON)
}

pub fn next_float_up(mut v: f32) -> f32 {
    if v == std::f32::INFINITY { return v; }

    if v == -0.0 { v = 0.0 }

    let bits = v.to_bits();
    let bits = if v >= 0.0 { bits + 1 } else { bits - 1 };
    f32::from_bits(bits)
}

pub fn next_float_down(mut v: f32) -> f32 {
    if v == std::f32::NEG_INFINITY { return v; }

    if v == 0.0 { v = -0.0 }

    let bits = v.to_bits();
    let bits = if v >= 0.0 { bits - 1 } else { bits + 1 };
    f32::from_bits(bits)
}