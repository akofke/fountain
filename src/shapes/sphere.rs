use crate::Float;
use crate::geometry::{Transform, Ray, Transformable};
use crate::shapes::Shape;
use crate::geometry::bounds::Bounds3;
use crate::interaction::SurfaceInteraction;

pub struct Sphere<'t> {
    object_to_world: &'t Transform,
    world_to_object: &'t Transform,
    reverse_orientation: bool,

    radius: Float,
    z_min: Float,
    z_max: Float,
    theta_min: Float,
    theta_max: Float,
    phi_max: Float
}

impl<'t> Sphere<'t> {
    pub fn new(
        object_to_world: &'t Transform,
        world_to_object: &'t Transform,
        reverse_orientation: bool,
        radius: Float,
        z_min: Float,
        z_max: Float,
        phi_max: Float
    ) -> Self {
        Self {
            object_to_world, world_to_object, reverse_orientation,
            radius,
            z_min: Float::min(z_min, z_max).clamp(-radius, radius),

            z_max: Float::max(z_min, z_max).clamp(-radius, radius),
            theta_min: Float::clamp(z_min / radius, -1.0, 1.0).acos(),
            theta_max: Float::clamp(z_max / radius, -1.0, 1.0).acos(),
            phi_max: phi_max.clamp(0.0, 360.0).to_radians()
        }
    }
}

impl<'t> Shape for Sphere<'t> {
    fn object_bound(&self) -> Bounds3<f32> {
        bounds3f!((-self.radius, -self.radius, self.z_min), (self.radius, self.radius, self.z_max))
    }

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)> {
        unimplemented!()
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        unimplemented!()
    }

    fn object_to_world<T: Transformable>(&self, t: T) -> T {
        unimplemented!()
    }

    fn world_to_object<T: Transformable>(&self, t: T) -> T {
        unimplemented!()
    }
}