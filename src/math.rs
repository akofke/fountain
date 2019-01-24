use num::clamp;
use num::traits::{Float};
use std::ops::{AddAssign, Add, Div, Mul, Sub};
use num::ToPrimitive;

pub type Vector3f = Vector3<f32>;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vector3<T>
where
    T: Float,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Float> Vector3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn to_rgb(&self) -> [u8; 3] where T: ToPrimitive {
        let zero = T::zero();
        let one = T::one();
        let scale = T::from(255).unwrap();
        [
            (clamp::<T>(self.x, zero, one) * scale).to_u8().unwrap(),
            (clamp::<T>(self.y, zero, one) * scale).to_u8().unwrap(),
            (clamp::<T>(self.z, zero, one) * scale).to_u8().unwrap(),
        ]
    }

    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// L2 norm
    pub fn norm(&self) -> T {
        (self.dot(self)).sqrt()
    }

    pub fn norm_squared(&self) -> T {
        self.dot(self)
    }

    pub fn normalize(&self) -> Self {
        *self * (T::one() / self.norm())
    }
}

impl<T: Float> AddAssign<T> for Vector3<T> {
    fn add_assign(&mut self, other: T) {
        *self = Vector3 {
            x: self.x + other,
            y: self.y + other,
            z: self.z + other,
        };
    }
}

impl<T: Float> AddAssign for Vector3<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        };
    }
}

impl<T: Float> Mul<T> for Vector3<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Vector3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs
        }
    }
}

impl<T: Float> Sub for Vector3<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}
