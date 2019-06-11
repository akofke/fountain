use crate::geometry::bounds::Bounds3f;
use crate::geometry::{Ray, Transformable};
use crate::interaction::SurfaceInteraction;
use crate::Float;


pub trait Shape {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f {
        unimplemented!()
    }

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)>;

    fn intersect_test(&self, ray: &Ray) -> bool;

    fn object_to_world<T: Transformable>(&self, t: T) -> T;

    fn world_to_object<T: Transformable>(&self, t: T) -> T;
}

pub struct ShapeBase {

}