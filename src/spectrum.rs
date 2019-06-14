use crate::Float;
use std::ops::{Add, Sub, AddAssign, SubAssign, Mul, MulAssign, Div, DivAssign, Index, IndexMut, Deref};

pub trait SpectrumArith: Add + AddAssign + Sub + SubAssign + Mul + MulAssign + Div + DivAssign + Mul<Float>
+ MulAssign<Float> + Div<Float> + DivAssign<Float>
    where Self: Sized {}

pub trait CoefficientSpectrum: Index<usize, Output=Float> + IndexMut<usize, Output=Float> + Copy {
    const N_SAMPLES: usize;

    fn new(v: Float) -> Self;

    fn sqrt(&self) -> Self {
        let mut res = Self::new(0.0);
        for i in 0..Self::N_SAMPLES {
            res[i] = self[i].sqrt();
        }
        res
    }
}

pub struct Spectrum<S: CoefficientSpectrum>(S);

impl<S: CoefficientSpectrum> Spectrum<S> {
    pub fn sqrt(&self) -> Self {
        let mut res = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            res[i] = self.0[i].sqrt();
        }
        Self(res)
    }

    pub fn lerp(t: Float, s1: Self, s2: Self) -> Self {
        (1.0 - t) * s1 + t * s2
    }

    pub fn clamp(&self, low: Float, high: Float) -> Self {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self.0[i].clamp(low, high);
        }
        Self(ret)
    }
}

impl<S: CoefficientSpectrum> Deref for Spectrum<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Clone, Copy, PartialEq)]
pub struct RGBSpectrum {
    c: [Float; 3]
}

impl RGBSpectrum {
}

impl CoefficientSpectrum for RGBSpectrum {
    const N_SAMPLES: usize = 3;

    fn new(v: Float) -> Self {
        Self {c: [v; 3]}
    }
}

impl Index<usize> for RGBSpectrum {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        &self.c[index]
    }
}

impl IndexMut<usize> for RGBSpectrum {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.c[index]
    }
}

impl<S> Add for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self.0[i] + rhs.0[i];
        }
        Self(ret)
    }
}

impl<S> AddAssign for Spectrum<S> where S: CoefficientSpectrum {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..S::N_SAMPLES {
            self.0[i] += rhs.0[i]
        }
    }
}

impl<S> Sub for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self.0[i] - rhs.0[i];
        }
        Self(ret)
    }
}

impl<S> SubAssign for Spectrum<S> where S: CoefficientSpectrum {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..S::N_SAMPLES {
            self.0[i] -= rhs.0[i]
        }
    }
}

impl<S> Mul for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self.0[i] * rhs.0[i];
        }
        Self(ret)
    }
}

impl<S> MulAssign for Spectrum<S> where S: CoefficientSpectrum {
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..S::N_SAMPLES {
            self.0[i] *= rhs.0[i]
        }
    }
}

impl<S> Div for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self.0[i] / rhs.0[i];
        }
        Self(ret)
    }
}

impl<S> DivAssign for Spectrum<S> where S: CoefficientSpectrum {
    fn div_assign(&mut self, rhs: Self) {
        for i in 0..S::N_SAMPLES {
            self.0[i] /= rhs.0[i]
        }
    }
}

impl<S> std::ops::Neg for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Self;

    fn neg(self) -> Self {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = -self.0[i]
        }
        Self(ret)
    }
}

impl<S> Mul<Spectrum<S>> for Float where S: CoefficientSpectrum {
    type Output = Spectrum<S>;

    fn mul(self, rhs: Spectrum<S>) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self * rhs.0[i];
        }
        Spectrum(ret)
    }
}
