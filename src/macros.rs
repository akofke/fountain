
/// Convenience macro that allows creating a Vec3 without needing to use f32 literals
///
/// ```
/// use raytracer::{vec3f, Vec3f};
/// assert_eq!(vec3f!(1, 2, 3), Vec3f::new(1.0, 2.0, 3.0));
/// ```
///
#[macro_export]
macro_rules! vec3f {
    ($x:expr, $y:expr, $z:expr) => {
        $crate::Vec3f::new($x as f32, $y as f32, $z as f32)
    };
}

#[macro_export]
macro_rules! point3f {
    ( ($x:expr , $y:expr , $z:expr) ) => { nalgebra::Point3::new($x as $crate::Float, $y as $crate::Float, $z as $crate::Float)};
    ($x:expr , $y:expr , $z:expr) => { nalgebra::Point3::new($x as $crate::Float, $y as $crate::Float, $z as $crate::Float)};
}

#[macro_export]
macro_rules! bounds3f {
    ( $p1:tt, $p2:tt ) => {
       $crate::Bounds3f::with_bounds($crate::point3f![$p1], $crate::point3f![$p2])
    };
}
