use num::clamp;
use num::traits::{Float};
use std::ops::AddAssign;
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
