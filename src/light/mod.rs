use crate::{Transform, Point2f, Vec3f, Float, RayDifferential};
use crate::interaction::SurfaceHit;
use crate::spectrum::Spectrum;
use crate::scene::Scene;
use crate::bvh::BVH;
use std::sync::Arc;
use crate::shapes::Shape;

pub mod point;
pub mod distant;
pub mod infinite;
pub mod diffuse;

pub trait Light: Sync + Send {
    fn flags(&self) -> LightFlags;

    fn light_to_world(&self) -> &Transform;

    fn world_to_light(&self) -> &Transform;

    fn n_samples(&self) -> usize { 1 }

    fn preprocess(&mut self, scene_prims: &BVH) {}

    fn sample_incident_radiance(&self, reference: &SurfaceHit, u: Point2f) -> LiSample;

    /// The probability density with respect to solid angle for the light's
    /// `sample_incident_radiance` method to sample the direction `wi` from the reference
    /// point `reference`.
    fn pdf_incident_radiance(&self, reference: &SurfaceHit, wi: Vec3f) -> Float;

    fn environment_emitted_radiance(&self, ray: &RayDifferential) -> Spectrum { Spectrum::new(0.0) }
}

pub trait AreaLight: Light {
    /// Given a point on the area light's surface represented by `hit`, evaluate the area light's
    /// emitted radiance `L` in the given outgoing direction `w`.
    fn emitted_radiance(&self, hit: SurfaceHit, w: Vec3f) -> Spectrum;

    // TODO: this is a hack for upcasting to compare pointers, which probably isn't even needed.
    fn as_light(&self) -> &dyn Light;
}

pub trait AreaLightBuilder<S: Shape> {
    type Target: AreaLight;

    fn create(self, shape: Arc<S>) -> Self::Target;
}

pub struct LiSample {
    pub radiance: Spectrum,

    /// The direction *towards* the illumination
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