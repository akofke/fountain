//TODO: uncomment when const generics don't crash

//
//use crate::Float;
//use std::ops::{Add, Sub, AddAssign, SubAssign, Mul, MulAssign, Div, DivAssign};
//
//pub trait SpectrumArith: Add + AddAssign + Sub + SubAssign + Mul + MulAssign + Div + DivAssign + Mul<Float>
//+ MulAssign<Float> + Div<Float> + DivAssign<Float>
//    where Self: Sized {}
//
//
//#[derive(Clone, Copy)]
//pub struct CoefficientSpectrum<const N: usize>
//{
//    pub c: [Float; N]
//}
//
//impl<const N: usize> Add for CoefficientSpectrum<{N}> {
//    type Output = Self;
//
//    fn add(self, rhs: Self) -> Self::Output {
//        let mut c = [0.0 as Float; N];
//        for i in 0..N {
//            c[i] = self.c[i] + rhs.c[i];
//        }
//        CoefficientSpectrum { c }
//    }
//}
//

