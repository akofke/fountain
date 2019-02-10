use nalgebra::Vector3;

pub type Vec3 = Vector3<f32>;

/// Convenience macro that allows creating a Vec3 without needing to use f32 literals
///
/// ```
/// assert_eq!(vec3!(1, 2, 3), Vec3::new(1.0, 2.0, 3.0));
/// ```
///
#[macro_export]
macro_rules! v3 {
    ($x:expr, $y:expr, $z:expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

pub fn to_array(v: Vec3) -> [f32; 3] {
    let mut arr = [0.0; 3];
    arr.copy_from_slice(v.as_slice());
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro() {
        assert_eq!(v3!(1, 2, 3), Vec3::new(1.0, 2.0, 3.0));
    }
}