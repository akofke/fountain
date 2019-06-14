

use crate::Float;
use std::ops::{Add, Sub, AddAssign, SubAssign, Mul, MulAssign, Div, DivAssign, Index};

pub trait SpectrumArith: Add + AddAssign + Sub + SubAssign + Mul + MulAssign + Div + DivAssign + Mul<Float>
+ MulAssign<Float> + Div<Float> + DivAssign<Float>
    where Self: Sized {}

pub trait CoefficientSpectrum<const N: usize>: Index<usize>  {

}

impl<S: CoefficientSpectrum<{N}>, const N: usize> Add for S {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        unimplemented!()
    }
}

pub struct RGBSpectrum {
    c: [Float; 3]
}

impl RGBSpectrum {
    pub fn new(v: Float) -> Self {
        Self {c: [v; 3]}
    }
}

impl CoefficientSpectrum<{3}> for RGBSpectrum {

}

impl Index<usize> for RGBSpectrum {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        &self.c[index]
    }
}
