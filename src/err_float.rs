use crate::Float;
use std::ops::{Add, Sub, Mul, Div, Neg};

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

#[derive(Clone, Copy)]
pub struct EFloat {
    pub v: Float,
    pub low: Float,
    pub high: Float,

    #[cfg(float_err_debug)]
    precise: f64
}

impl EFloat {
    pub fn new(v: Float) -> Self {
        Self::with_err(v, 0.0)
    }

    pub fn with_err(v: Float, err: Float) -> Self {
        if err == 0.0 {
            return Self::with_bounds(v, v, v)
        }
        Self {
            v,
            low: next_float_down(v - err),
            high: next_float_up(v + err),
            #[cfg(float_err_debug)] precise: v as f64
        }
    }

    pub fn with_bounds(v: Float, low: Float, high: Float) -> Self {
        Self {
            v, low, high,
            #[cfg(float_err_debug)] precise: v as f64
        }
    }

    pub fn absolute_err(&self) -> Float {
        self.high - self.low
    }


    pub fn upper_bound(&self) -> Float {
        self.high
    }

    pub fn lower_bound(&self) -> Float {
        self.low
    }

    pub fn sqrt(&self) -> Self {
        let v = self.v.sqrt();
        let low = next_float_down(self.low.sqrt());
        let high = next_float_up(self.high.sqrt());
        Self::with_bounds(v, low, high)
    }

    pub fn abs(self) -> Self {
        if self.low >= 0.0 { return self }

        if self.high <= 0.0 { return -self }

        let v = self.v.abs();
        let low = 0.0f32;
        let high = Float::max(-self.low, self.high);
        Self::with_bounds(v, low, high)
    }
}

impl From<EFloat> for Float {
    fn from(f: EFloat) -> Self {
        f.v
    }
}

impl PartialEq for EFloat {
    fn eq(&self, other: &EFloat) -> bool {
        self.v == other.v
    }
}

impl Add for EFloat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let v = self.v + rhs.v;
        let low = next_float_down(self.lower_bound() + rhs.lower_bound());
        let high = next_float_up(self.upper_bound() + rhs.upper_bound());
        Self::with_bounds(v, low, high)
    }
}

impl Sub for EFloat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let v = self.v - rhs.v;
        let low = next_float_down(self.lower_bound() - rhs.lower_bound());
        let high = next_float_up(self.upper_bound() - rhs.upper_bound());
        Self::with_bounds(v, low, high)
    }
}

impl Mul for EFloat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let v = self.v * rhs.v;
        let prod1 = self.lower_bound() * rhs.lower_bound();
        let prod2 = self.upper_bound() * rhs.lower_bound();
        let prod3 = self.lower_bound() * rhs.upper_bound();
        let prod4 = self.upper_bound() * rhs.upper_bound();

        let low = next_float_down(Float::min(
            Float::min(prod1, prod2),
            Float::min(prod3, prod4)
        ));
        let high = next_float_up(Float::max(
            Float::max(prod1, prod2),
            Float::max(prod3, prod4)
        ));
        Self::with_bounds(v, low, high)
    }
}

impl Div for EFloat {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let v = self.v / rhs.v;

        if rhs.low < 0.0 && rhs.high > 0.0 {
            return Self::with_bounds(v, std::f32::NEG_INFINITY, std::f32::INFINITY)
        }

        let div1 = self.lower_bound() / rhs.lower_bound();
        let div2 = self.upper_bound() / rhs.lower_bound();
        let div3 = self.lower_bound() / rhs.upper_bound();
        let div4 = self.upper_bound() / rhs.upper_bound();

        let low = next_float_down(Float::min(
            Float::min(div1, div2),
            Float::min(div3, div4)
        ));
        let high = next_float_up(Float::max(
            Float::max(div1, div2),
            Float::max(div3, div4)
        ));
        Self::with_bounds(v, low, high)
    }
}

impl Neg for EFloat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::with_bounds(-self.v, -self.high, -self.low)
    }
}

impl Add<EFloat> for Float {
    type Output = EFloat;

    fn add(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self) + rhs
    }
}

impl Sub<EFloat> for Float {
    type Output = EFloat;

    fn sub(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self) - rhs
    }
}

impl Mul<EFloat> for Float {
    type Output = EFloat;

    fn mul(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self) * rhs
    }
}

impl Div<EFloat> for Float {
    type Output = EFloat;

    fn div(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self) / rhs
    }
}
