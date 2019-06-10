use nalgebra::Vector3;
use nalgebra::Transform3;
use crate::Vec3f;


pub fn to_array(v: Vec3f) -> [f32; 3] {
    let mut arr = [0.0; 3];
    arr.copy_from_slice(v.as_slice());
    arr
}

