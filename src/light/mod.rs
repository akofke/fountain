use crate::{Transform, Point2f, Vec3f, Float};
use crate::interaction::SurfaceHit;
use crate::spectrum::Spectrum;
use crate::scene::Scene;

pub trait Light {
    const FLAGS: LightFlags;

    fn light_to_world(&self) -> &Transform;

    fn world_to_light(&self) -> &Transform;

    fn n_samples(&self) -> usize { 1 }

    fn sample_incident_radiance(&self, reference: &SurfaceHit, u: Point2f) -> LiSample;
}

pub struct LiSample {
    pub radiance: Spectrum,
    pub wi: Vec3f,
    pub pdf: Float,
    pub vis: VisibilityTester,
}

pub enum LightFlags {
    DeltaPosition, DeltaDirection, Area, Infinite
}

impl LightFlags {
    pub fn is_delta_light(&self) -> bool {
        match self {
            LightFlags::DeltaDirection | LightFlags::DeltaPosition => true,
            _ => false
        }
    }
}

pub struct VisibilityTester {
    pub p0: SurfaceHit,
    pub p1: SurfaceHit,
}

impl VisibilityTester {
    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_test(&self.p0.spawn_ray_to_hit(self.p1))
    }
}