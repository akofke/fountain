use crate::bvh::BVH;
use crate::{SurfaceInteraction, Ray};

pub struct Scene<'p> {
    pub primitives_aggregate: BVH<'p>
}

impl Scene<'_> {
    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.primitives_aggregate.intersect(ray)
    }

    pub fn intersect_test(&self, ray: &Ray) -> bool {
        self.primitives_aggregate.intersect_test(ray)
    }
}