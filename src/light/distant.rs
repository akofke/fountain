use cgmath::InnerSpace;
use num::Zero;

use crate::{Float, Normal3, Point2f, Point3f, Transform, Vec3f};
use crate::bvh::BVH;
use crate::interaction::SurfaceHit;
use crate::light::{Light, LightFlags, LiSample, VisibilityTester};
use crate::scene::Scene;
use crate::spectrum::Spectrum;

pub struct DistantLight {
    radiance: Spectrum,
    dir_to_light: Vec3f,
    world_center: Point3f,
    world_radius: Float,
}

impl DistantLight {
    pub fn from_to(from: Point3f, to: Point3f, radiance: Spectrum) -> Self {
        Self::new(radiance, from - to)
    }
    pub fn new(radiance: Spectrum, dir_to_light: Vec3f) -> Self {
        let dir_to_light = dir_to_light.normalize();
        Self {
            radiance,
            dir_to_light,
            world_center: Point3f::new(0.0, 0.0, 0.0),
            world_radius: 0.0,
        }
    }
}

impl Light for DistantLight {
    fn flags(&self) -> LightFlags {
        LightFlags::DeltaDirection
    }

    fn light_to_world(&self) -> &Transform {
        &Transform::IDENTITY
    }

    fn world_to_light(&self) -> &Transform {
        &Transform::IDENTITY
    }

    fn preprocess(&mut self, scene_prims: &BVH) {
        let (world_center, world_radius) = scene_prims.bounds.bounding_sphere();
        self.world_center = world_center;
        self.world_radius = world_radius;
    }

    fn sample_incident_radiance(&self, reference: &SurfaceHit, _u: Point2f) -> LiSample {
        let p_outside = reference.p + self.dir_to_light * (2.0 * self.world_radius);

        let p1 = SurfaceHit {
            p: p_outside,
            p_err: Vec3f::zero(),
            time: reference.time,
            n: Normal3(Vec3f::zero()),
        };

        let vis = VisibilityTester {
            p0: *reference,
            p1,
        };

        LiSample {
            radiance: self.radiance,
            wi: self.dir_to_light,
            pdf: 1.0,
            vis,
        }
    }
}