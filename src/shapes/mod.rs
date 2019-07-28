use crate::geometry::bounds::Bounds3f;
use crate::geometry::{Ray, Transformable};
use crate::interaction::SurfaceInteraction;
use crate::Float;

pub mod sphere;

pub trait Shape: Sync + Send {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f where Self: Sized {
        self.object_to_world(self.object_bound())
    }

    fn object_to_world<T: Transformable<O>, O>(&self, t: T) -> O where Self: Sized;

    fn world_to_object<T: Transformable<O>, O>(&self, t: T) -> O where Self: Sized;

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)>;

    fn intersect_test(&self, ray: &Ray) -> bool;

}

pub struct ShapeBase {

}
