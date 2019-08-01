use crate::bvh::BVH;
use crate::{SurfaceInteraction, Ray};
use crate::light::Light;

pub struct Scene<'p> {
    pub primitives_aggregate: BVH<'p>,
    pub lights: Vec<&'p dyn Light>,
}

impl Scene<'_> {
    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.primitives_aggregate.intersect(ray)
    }

    pub fn intersect_test(&self, ray: &Ray) -> bool {
        self.primitives_aggregate.intersect_test(ray)
    }
}