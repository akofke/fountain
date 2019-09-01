use crate::{Float, Transform};
use crate::geometry::Ray;
use crate::geometry::bounds::Bounds3f;
use crate::interaction::SurfaceInteraction;

pub mod sphere;
pub mod triangle;

pub trait Shape: Sync + Send {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f {
        self.object_to_world().transform(self.object_bound())
    }

    fn object_to_world(&self) -> &Transform;

    fn world_to_object(&self) -> &Transform;

    fn transform_swaps_handedness(&self) -> bool {
        self.object_to_world().swaps_handedness()
    }

    fn reverse_orientation(&self) -> bool;

    fn flip_normals(&self) -> bool {
        self.reverse_orientation() ^ self.transform_swaps_handedness()
    }

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)>;

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

}
