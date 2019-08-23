use std::mem::MaybeUninit;

use crate::Float;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CoefficientSpectrum<const N: usize>([Float; N]);

impl<const N: usize> CoefficientSpectrum<{N}> {

    #[inline]
    pub fn new_with(init: impl Fn(usize) -> Float) -> Self {
        let mut arr: MaybeUninit<Self> = MaybeUninit::uninit();
        let arr_pointer: *mut Float = unsafe {
            // Safe since Self is repr(transparent).
            std::mem::transmute(&mut arr)
        };

        for i in 0..N {
            unsafe {
                // Safe since i is always inbounds
                arr_pointer.add(i).write(init(i));
            }
        }

        // Safe since we have initialized every place in the array
        unsafe { arr.assume_init() }
    }

    #[inline]
    pub fn zip<F: Fn(Float, Float) -> Float>(&self, other: &Self, f: F) -> Self {
        Self::new_with(|i| f(self.0[i], other.0[i]))
    }
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

impl<const N: usize> std::ops::Add for CoefficientSpectrum<{N}> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip(&rhs, |x, y| x + y)
    }
}

impl<const N: usize> std::ops::AddAssign for CoefficientSpectrum<{N}> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.0[i] += rhs.0[i];
        }
    }
}

impl<const N: usize> std::ops::Sub for CoefficientSpectrum<{N}> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip(&rhs, |x, y| x - y)
    }
}

impl<const N: usize> std::ops::SubAssign for CoefficientSpectrum<{N}> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.0[i] -= rhs.0[i];
        }
    }
}

impl<const N: usize> std::ops::Mul for CoefficientSpectrum<{N}> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip(&rhs, |x, y| x * y)
    }
}

impl<const N: usize> std::ops::MulAssign for CoefficientSpectrum<{N}> {
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.0[i] *= rhs.0[i];
        }
    }
}

impl<const N: usize> std::ops::Div for CoefficientSpectrum<{N}> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.zip(&rhs, |x, y| x / y)
    }
}

impl<const N: usize> std::ops::DivAssign for CoefficientSpectrum<{N}> {
    fn div_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.0[i] /= rhs.0[i];
        }
    }
}


impl<const N: usize> std::ops::Add<Float> for CoefficientSpectrum<{N}> {
    type Output = Self;

    fn add(self, rhs: Float) -> Self::Output {
        Self::new_with(|i| self[i] * rhs)
    }
}

impl<const N: usize> std::ops::AddAssign<Float> for CoefficientSpectrum<{N}> {
    fn add_assign(&mut self, rhs: Float) {
        for i in 0..N {
            self[i] += rhs;
        }
    }
}
