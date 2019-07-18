use crate::bvh::BVH;
use crate::{SurfaceInteraction, Ray};

pub struct Scene {
    primitives_aggregate: BVH
}

impl Scene {
    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.primitives_aggregate.intersect(ray)
    }
}