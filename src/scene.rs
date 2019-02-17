use crate::geometry::Sphere;
use crate::aabb::Aabb;

pub struct PrimId(usize);

pub struct PrimRef {
    pub aabb: Aabb,
    pub idx: usize
}

pub struct Scene {
    pub spheres: Vec<Sphere>,

    prim_bounding_boxes: Vec<Aabb>
}

impl Scene {
    pub fn new(spheres: Vec<Sphere>) -> Self {
        Self {
            spheres,

            prim_bounding_boxes: Vec::new()
        }
    }
}