use crate::{Transform, Point3f, Float, Point2f, Vec3f, Normal3};
use crate::spectrum::Spectrum;
use crate::light::{Light, LightFlags, LiSample, VisibilityTester};
use crate::interaction::SurfaceHit;
use cgmath::{InnerSpace, MetricSpace};
use num::Zero;

pub struct PointLight {
    l2w: Transform,
    w2l: Transform,
    world_point: Point3f,
    intensity: Spectrum,
}

impl PointLight {
    pub fn new(light_to_world: Transform, intensity: Spectrum) -> Self {
        let l2w = light_to_world;
        let w2l = l2w.inverse();
        let world_point = l2w.transform(Point3f::new(0.0, 0.0, 0.0));
        Self {
            l2w,
            w2l,
            world_point,
            intensity
        }
    }
}

impl Light for PointLight {
    fn flags(&self) -> LightFlags {
        LightFlags::DeltaPosition
    }

    fn light_to_world(&self) -> &Transform {
        &self.l2w
    }

    fn world_to_light(&self) -> &Transform {
        &self.w2l
    }

    fn sample_incident_radiance(&self, reference: &SurfaceHit, u: Point2f) -> LiSample {
        let wi = (self.world_point - reference.p).normalize();
        let pdf = 1.0;
        let p1 = SurfaceHit {
            p: self.world_point,
            p_err: Vec3f::zero(),
            time: reference.time,
            n: Normal3(Vec3f::zero()),
        };
        let vis = VisibilityTester {
            p0: *reference,
            p1,
        };
        let radiance = self.intensity / (self.world_point - reference.p).magnitude2();
        LiSample {
            radiance,
            wi,
            vis,
            pdf
        }
    }
}