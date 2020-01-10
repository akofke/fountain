use crate::bvh::BVH;
use crate::{SurfaceInteraction, Ray, Bounds3f};
use crate::light::Light;
use std::sync::Arc;
use crate::primitive::Primitive;
use crate::shapes::triangle::TriangleMesh;
use std::fmt::{Debug, Formatter};

pub struct SceneBuilder {

}

pub struct Scene {
    pub primitives_aggregate: BVH,
    pub lights: Vec<Arc<dyn Light>>,
    pub meshes: Vec<Arc<TriangleMesh>>,
}

impl Debug for Scene {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let n_prims = self.primitives_aggregate.prims.len();
        let scene_bounds = self.primitives_aggregate.bounds;
        let n_lights = self.lights.len();
        let n_meshes = self.meshes.len();
        writeln!(f, "Scene{{ {} prims, {} lights, {} meshes, bounds {:?} }}", n_prims, n_lights, n_meshes, scene_bounds)
    }
}

impl Scene {

    pub fn new(primitives: BVH, mut lights: Vec<Arc<dyn Light>>, meshes: Vec<Arc<TriangleMesh>>) -> Self {
        // TODO: this is kind of weird, maybe find a better way to do preprocess
        for light in &mut lights {
            Arc::get_mut(light).unwrap().preprocess(&primitives);
        }

        for prim in &primitives.prims {
            if let Some(light) = prim.light_arc_cloned() {
                lights.push(light)
            }
        }

        Self {
            primitives_aggregate: primitives,
            lights,
            meshes
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