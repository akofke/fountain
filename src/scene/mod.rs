use crate::bvh::BVH;
use crate::{SurfaceInteraction, Ray, Bounds3f};
use crate::light::Light;

pub struct SceneBuilder {

}

pub struct Scene<'p> {
    pub primitives_aggregate: BVH<'p>,
    pub lights: Vec<&'p dyn Light>,
}

impl<'p> Scene<'p> {

    pub fn new(primitives: BVH<'p>, lights: Vec<&'p dyn Light>) -> Self {
        // TODO: this is kind of weird, maybe find a better way to do preprocess
        let lights = lights.into_iter()
            .map(|light| {
                light.preprocess(&primitives);
                &*light
            })
            .collect();

        Self {
            primitives_aggregate: primitives,
            lights
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.primitives_aggregate.intersect(ray)
    }

    pub fn intersect_test(&self, ray: &Ray) -> bool {
        self.primitives_aggregate.intersect_test(ray)
    }

    pub fn world_bound(&self) -> Bounds3f {
        self.primitives_aggregate.bounds
    }
}