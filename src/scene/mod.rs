use crate::bvh::BVH;
use crate::{SurfaceInteraction, Ray, Bounds3f};
use crate::light::Light;
use std::sync::Arc;
use crate::primitive::Primitive;

pub struct SceneBuilder {

}

pub struct Scene {
    pub primitives_aggregate: BVH,
    pub lights: Vec<Box<dyn Light>>,
}

impl Scene {

    pub fn new(primitives: BVH, mut lights: Vec<Box<dyn Light>>) -> Self {
        // TODO: this is kind of weird, maybe find a better way to do preprocess
        for light in &mut lights {
            light.preprocess(&primitives);
        }

        for prim in &primitives.prims {
            if let Some(light) = prim.area_light() {
                lights.push(light.as_light())
            }
        }

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