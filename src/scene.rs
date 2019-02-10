use crate::geometry::Sphere;

pub struct Scene {
    pub spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new(spheres: Vec<Sphere>) -> Self {
        Self {
            spheres
        }
    }
}