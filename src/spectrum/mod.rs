use crate::Float;
use std::ops::{Add, Sub, AddAssign, SubAssign, Mul, MulAssign, Div, DivAssign, Index, IndexMut, Deref};

pub fn xyz_to_rgb(xyz: [Float; 3]) -> [Float; 3] {
    let mut rgb = [0.0; 3];
    rgb[0] =  3.240479*xyz[0] - 1.537150*xyz[1] - 0.498535*xyz[2];
    rgb[1] = -0.969256*xyz[0] + 1.875991*xyz[1] + 0.041556*xyz[2];
    rgb[2] =  0.055648*xyz[0] - 0.204043*xyz[1] + 1.057311*xyz[2];
    rgb
}

pub fn rgb_to_xyz(rgb: [Float; 3]) -> [Float; 3] {
    let mut xyz = [0.0; 3];
    xyz[0] = 0.412453*rgb[0] + 0.357580*rgb[1] + 0.180423*rgb[2];
    xyz[1] = 0.212671*rgb[0] + 0.715160*rgb[1] + 0.072169*rgb[2];
    xyz[2] = 0.019334*rgb[0] + 0.119193*rgb[1] + 0.950227*rgb[2];
    xyz
}

pub trait CoefficientSpectrum: Index<usize, Output=Float> + IndexMut<usize, Output=Float> + Copy {
    const N_SAMPLES: usize;

    fn new(v: Float) -> Self;

    fn to_xyz(&self) -> [Float; 3];

    fn to_rgb(&self) -> [Float; 3];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Spectrum<S: CoefficientSpectrum=RGBSpectrum>(S);

impl<S: CoefficientSpectrum> Spectrum<S> {
    pub fn new(v: Float) -> Self {
        Self(S::new(v))
    }
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

    pub fn clamp_positive(&self) -> Self {
        self.clamp(0.0, std::f32::INFINITY)
    }

    pub fn is_black(&self) -> bool {
        for i in 0..S::N_SAMPLES {
            if self.0[i] != 0.0 { return false; }
        }
        true
    }

    pub fn has_nans(&self) -> bool {
        for i in 0..S::N_SAMPLES {
            if self.0[i].is_nan() { return true }
        }
        false
    }
}

impl<S: CoefficientSpectrum> std::iter::Sum for Spectrum<S> {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Self::new(0.0), Add::add)
    }
}

impl From<Spectrum<RGBSpectrum>> for [Float; 3] {
    fn from(s: Spectrum<RGBSpectrum>) -> Self {
        s.c
    }
}

impl From<[Float; 3]> for Spectrum<RGBSpectrum> {
    fn from(c: [Float; 3]) -> Self {
        Self(RGBSpectrum{ c })
    }
}

impl<S: CoefficientSpectrum> Deref for Spectrum<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Clone, Copy, PartialEq, Debug, Default)]
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

    fn to_xyz(&self) -> [Float; 3] {
        rgb_to_xyz(self.c)
    }

    fn to_rgb(&self) -> [Float; 3] {
        self.c
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

//
// Spectrum (op) Spectrum
//

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

//
// Float (op) Spectrum
//

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

impl<S> Add<Spectrum<S>> for Float where S: CoefficientSpectrum {
    type Output = Spectrum<S>;

    fn add(self, rhs: Spectrum<S>) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self + rhs.0[i];
        }
        Spectrum(ret)
    }
}

//
// Spectrum (op) Float
//

impl<S> Mul<Float> for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Spectrum<S>;

    fn mul(self, rhs: Float) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self[i] * rhs;
        }
        Spectrum(ret)
    }
}

impl<S> Div<Float> for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Spectrum<S>;

    fn div(self, rhs: Float) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self[i] / rhs;
        }
        Spectrum(ret)
    }
}

impl<S> Sub<Float> for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Spectrum<S>;

    fn sub(self, rhs: Float) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self[i] - rhs;
        }
        Spectrum(ret)
    }
}

impl<S> Add<Float> for Spectrum<S> where S: CoefficientSpectrum {
    type Output = Spectrum<S>;

    fn add(self, rhs: Float) -> Self::Output {
        let mut ret = S::new(0.0);
        for i in 0..S::N_SAMPLES {
            ret[i] = self[i] + rhs;
        }
        Spectrum(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_sum() {
        let spectra = vec![Spectrum::new(1.0), Spectrum::from([0.0, 1.0, 0.5])];
        let sum: Spectrum = spectra.into_iter().sum();
        assert_eq!(sum, Spectrum::from([1.0, 2.0, 1.5]));
    }
}
