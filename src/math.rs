use num::traits::{Float};
use std::ops::AddAssign;

pub type Vector3f = Vector3<f32>;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vector3<T> where T: Float {
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T: Float> Vector3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z
        }
    }
}

impl<T: Float> AddAssign<T> for Vector3<T> {

    fn add_assign(&mut self, other: T) {
        *self = Vector3 {x: self.x + other, y: self.y + other, z: self.z + other};
    }
}

impl<T: Float> AddAssign for Vector3<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = Vector3 {x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z};
    }
}

