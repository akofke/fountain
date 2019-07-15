use crate::geometry::bounds::Bounds3f;
use crate::{Ray, SurfaceInteraction};

pub trait Primitive {
    fn world_bound(&self) -> Bounds3f;

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction>;

    fn intersect_test(&self, ray: &Ray) -> bool;

    // TODO
}