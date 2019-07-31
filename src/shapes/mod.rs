use crate::geometry::bounds::Bounds3f;
use crate::geometry::{Ray, Transformable};
use crate::interaction::SurfaceInteraction;
use crate::{Float, Transform};

pub mod sphere;

pub trait Shape: Sync + Send {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f {
        self.object_to_world().transform(self.object_bound())
    }

    fn object_to_world(&self) -> &Transform;

    fn world_to_object(&self) -> &Transform;

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)>;

    fn intersect_test(&self, ray: &Ray) -> bool;

}

pub struct ShapeBase {

}
