use crate::geometry::bounds::Bounds3f;
use crate::{Ray, SurfaceInteraction};
use crate::material::Material;
use std::rc::Rc;
use crate::shapes::Shape;
use std::borrow::Borrow;
use std::sync::Arc;

pub trait Primitive: Sync {
    fn world_bound(&self) -> Bounds3f;

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction>;

    fn intersect_test(&self, ray: &Ray) -> bool;

    fn material(&self) -> Option<&dyn Material>;
}

pub struct GeometricPrimitive<S: Shape> {
    pub shape: S,  // TODO: use generic param instead?
    pub material: Option<Arc<dyn Material>>,

}

impl<'a, S: Shape> Primitive for GeometricPrimitive<S> {
    fn world_bound(&self) -> Bounds3f {
        self.shape.world_bound()
    }

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        let (t_hit, mut intersect) = self.shape.intersect(ray)?;

        ray.t_max = t_hit;
        intersect.primitive = Some(self); // TODO: this is terrible
        Some(intersect)
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray)
    }

    fn material(&self) -> Option<&dyn Material> {
        self.material.as_ref().map(|m| m.as_ref()) // ugly?
//        match &self.material {
//            Some(mat) => Some(mat.as_ref()),
//            None => None
//        }
    }
}