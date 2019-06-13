use crate::geometry::bounds::Bounds3f;
use crate::geometry::{Ray, Transformable};
use crate::interaction::SurfaceInteraction;
use crate::Float;

pub mod sphere;

pub trait Shape {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f {
        unimplemented!()
    }

    fn object_to_world<T: Transformable<O>, O>(&self, t: T) -> O;

    fn world_to_object<T: Transformable<O>, O>(&self, t: T) -> O;

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)>;

    fn intersect_test(&self, ray: &Ray) -> bool;

}

pub struct ShapeBase {

}
