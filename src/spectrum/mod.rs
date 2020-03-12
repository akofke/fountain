use std::mem::MaybeUninit;

use crate::Float;

pub fn array<F: FnMut(usize) -> Float, const N: usize>(mut init: F) -> [Float; N] {
    let mut arr = MaybeUninit::<[Float; N]>::uninit();
    let arr_pointer = arr.as_mut_ptr() as *mut Float;

    for i in 0..N {
        unsafe {
            // Safe since i is always inbounds
            arr_pointer.add(i).write(init(i));
        }
    }

    // Safe since we have initialized every place in the array
    let arr = unsafe { arr.assume_init() };
    arr
}

pub fn zip<F: Fn(Float, Float) -> Float, const N: usize>(a: [Float; N], b: [Float; N], f: F) -> [Float; N] {
    array(|i| f(a[i], b[i]))
}

#[allow(clippy::excessive_precision)]
pub fn xyz_to_rgb(xyz: [Float; 3]) -> [Float; 3] {
    let mut rgb = [0.0; 3];
    rgb[0] = 3.240479 * xyz[0] - 1.537150 * xyz[1] - 0.498535 * xyz[2];
    rgb[1] = -0.969256 * xyz[0] + 1.875991 * xyz[1] + 0.041556 * xyz[2];
    rgb[2] = 0.055648 * xyz[0] - 0.204043 * xyz[1] + 1.057311 * xyz[2];
    rgb
}

#[allow(clippy::excessive_precision)]
pub fn rgb_to_xyz(rgb: [Float; 3]) -> [Float; 3] {
    let mut xyz = [0.0; 3];
    xyz[0] = 0.412453 * rgb[0] + 0.357580 * rgb[1] + 0.180423 * rgb[2];
    xyz[1] = 0.212671 * rgb[0] + 0.715160 * rgb[1] + 0.072169 * rgb[2];
    xyz[2] = 0.019334 * rgb[0] + 0.119193 * rgb[1] + 0.950227 * rgb[2];
    xyz
}

#[derive(Clone, Copy)]
pub struct CoefficientSpectrum<const N: usize>([Float; N]);

pub type Spectrum = CoefficientSpectrum<{3}>;

impl<const N: usize> CoefficientSpectrum<{N}> {

    #[inline]
    pub fn new_with<F: FnMut(usize) -> Float>(init: F) -> Self {
        let arr = array(init);
        Self(arr)
    }

    #[inline]
    pub fn zip<F: Fn(Float, Float) -> Float>(&self, other: &Self, f: F) -> Self {
        let arr = zip(self.0, other.0, f);
        Self(arr)
    }

    pub fn uniform(val: Float) -> Self {
        Self::new_with(|_| val)
    }

    pub fn map<F: Fn(Float) -> Float>(&self, f: F) -> Self {
        Self::new_with(|i| f(self[i]))
    }

    pub fn is_black(&self) -> bool {
        self.0.iter().all(|&x| x == 0.0)
    }

    pub fn has_nans(&self) -> bool {
        self.0.iter().any(|&x| x.is_nan())
    }

    pub fn lerp(t: Float, s1: Self, s2: Self) -> Self {
        (1.0 - t) * s1 + t * s2
    }

    pub fn sqrt(self) -> Self {
        Self::new_with(|i| self[i].sqrt())
    }

    pub fn clamp(self, low: Float, high: Float) -> Self {
        Self::new_with(|i| self[i].clamp(low, high))
    }

    pub fn clamp_positive(self) -> Self {
        self.clamp(0.0, std::f32::INFINITY)
    }

    // FIXME: These have weird hacks and aren't implemented for <3> as a workaround for
    //  rustc stack overflow (https://github.com/rust-lang/rust/issues/68104).
    //  Revert when that is fixed.
    pub fn to_xyz(self) -> [Float; 3] {
        assert_eq!(N, 3);
        let mut arr = [0.0; 3];
        for i in 0..N {
            arr[i] = self[i]
        }
        rgb_to_xyz(arr)
    }

    pub fn to_rgb(self) -> [Float; 3] {
        assert_eq!(N, 3);
        let mut arr = [0.0; 3];
        for i in 0..N {
            arr[i] = self[i]
        }
        arr
//        self.0
    }
}

pub fn spectrum_from_rgb8(rgb8: [u8; 3]) -> Spectrum {
    let c = [
        rgb8[0] as Float / 255.0,
        rgb8[1] as Float / 255.0,
        rgb8[2] as Float / 255.0,
    ];
    CoefficientSpectrum(c)
}

pub fn spectrum_into_rgb8(s: Spectrum) -> [u8; 3] {
    let rgb = [
        Float::round(s[0] * 255.0) as u8,
        Float::round(s[1] * 255.0) as u8,
        Float::round(s[2] * 255.0) as u8,
    ];
    rgb
}

impl CoefficientSpectrum<{3}> {
    // pub fn to_xyz(self) -> [Float; 3] {
   //     rgb_to_xyz(self.0)
   // }
   //
   // pub fn to_rgb(self) -> [Float; 3] {
   //     self.0
   // }

    // pub fn from_rgb8(rgb8: [u8; 3]) -> Self {
    //     let c = [
    //         rgb8[0] as Float / 255.0,
    //         rgb8[1] as Float / 255.0,
    //         rgb8[2] as Float / 255.0,
    //     ];
    //     Self(c)
    // }
    //
    // pub fn into_rgb8(self) -> [u8; 3] {
    //     let rgb = [
    //         Float::round(self[0] * 255.0) as u8,
    //         Float::round(self[1] * 255.0) as u8,
    //         Float::round(self[2] * 255.0) as u8,
    //     ];
    //     rgb
    // }
}

impl<const N: usize> std::ops::Index<usize> for CoefficientSpectrum<{N}> {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize> std::ops::IndexMut<usize> for CoefficientSpectrum<{N}> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize> std::cmp::PartialEq for CoefficientSpectrum<{N}> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if self[i] != other[i] {
                return false
            }
        }
        true
    }
}

impl<const N: usize> Default for CoefficientSpectrum<{N}> {
    fn default() -> Self {
        Self::uniform(Float::default())
    }
}

impl<const N: usize> std::fmt::Debug for CoefficientSpectrum<{N}> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<const N: usize> From<[Float; N]> for CoefficientSpectrum<{N}> {
    fn from(a: [Float; N]) -> Self {
        Self(a)
    }
}

impl<const N: usize> From<Float> for CoefficientSpectrum<{N}> {
    fn from(x: Float) -> Self {
        Self::uniform(x)
    }
}

impl<const N: usize> From<CoefficientSpectrum<{N}>> for [Float; N] {
    fn from(s: CoefficientSpectrum<{N}>) -> Self {
        s.0
    }
}

impl<const N: usize> std::iter::Sum for CoefficientSpectrum<{N}> {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Self::uniform(0.0), std::ops::Add::add)
    }
}

impl<const N: usize> std::ops::Neg for CoefficientSpectrum<{N}> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new_with(|i| -self[i])
    }
}

macro_rules! impl_op {
    ($op:ident, $name:ident, $sym:tt) => {
        impl<const N: usize> std::ops::$op for CoefficientSpectrum<{N}> {
            type Output = Self;

            fn $name(self, rhs: Self) -> Self::Output {
                Self::zip(&self, &rhs, |x, y| x $sym y)
            }
        }

        impl<const N: usize> std::ops::$op<Float> for CoefficientSpectrum<{N}> {
            type Output = Self;

            fn $name(self, rhs: Float) -> Self::Output {
                Self::new_with(|i| self[i] $sym rhs)
            }
        }

        impl<const N: usize> std::ops::$op<CoefficientSpectrum<{N}>> for Float {
            type Output = CoefficientSpectrum<{N}>;

            fn $name(self, rhs: CoefficientSpectrum<{N}>) -> Self::Output {
                CoefficientSpectrum::new_with(|i| self $sym rhs[i])
            }
        }
    }
}

macro_rules! impl_assign_op {
    ($op:ident, $name:ident, $sym:tt) => {
        impl<const N: usize> std::ops::$op for CoefficientSpectrum<{N}> {
            fn $name(&mut self, rhs: Self) {
                for i in 0..N {
                    self[i] $sym rhs[i];
                }
            }
        }

        impl<const N: usize> std::ops::$op<Float> for CoefficientSpectrum<{N}> {
            fn $name(&mut self, rhs: Float) {
                for i in 0..N {
                    self[i] $sym rhs;
                }
            }
        }
    }
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);
impl_assign_op!(AddAssign, add_assign, +=);
impl_assign_op!(SubAssign, sub_assign, -=);
impl_assign_op!(MulAssign, mul_assign, *=);
impl_assign_op!(DivAssign, div_assign, /=);


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_iter_sum() {
        let spectra = vec![Spectrum::uniform(1.0), Spectrum::from([0.0, 1.0, 0.5])];
        let sum: Spectrum = spectra.into_iter().sum();
        assert_eq!(sum, Spectrum::from([1.0, 2.0, 1.5]));
    }
}
