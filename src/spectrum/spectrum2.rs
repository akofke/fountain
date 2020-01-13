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

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CoefficientSpectrum<const N: usize>([Float; N]);

impl<const N: usize> CoefficientSpectrum<{N}> {

    #[inline]
    pub fn new_with<F: FnMut(usize) -> Float>(mut init: F) -> Self {
        let arr = array(init);
        Self(arr)
    }

    #[inline]
    pub fn zip<F: Fn(Float, Float) -> Float>(&self, other: &Self, f: F) -> Self {
        let arr = zip(self.0, other.0, f);
        Self(arr)
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
        let arr = zip(self.0, rhs.0, |x, y| x + y);
        Self(arr)
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

#[cfg(test)]
mod tests {
    use super::*;


}
