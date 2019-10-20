use crate::{Float, Transform, Vec3f};
use crate::spectrum::{Spectrum, RGBSpectrum};
use std::rc::Rc;
use crate::shapes::Shape;
use crate::light::{AreaLight, Light, LiSample, LightFlags, VisibilityTester};
use crate::interaction::SurfaceHit;
use cgmath::{Vector3, InnerSpace, Point2};

pub struct DiffuseAreaLight<'s> {
    emit: Spectrum,
    shape: &'s dyn Shape,
    l2w: Transform,
    w2l: Transform,
    area: Float,
    n_samples: usize
}

impl<'s> DiffuseAreaLight<'s> {
    pub fn new(emit: Spectrum, shape: &'s dyn Shape, light_to_world: Transform, n_samples: usize) -> Self {
        Self {
            emit,
            shape,
            l2w: light_to_world,
            w2l: light_to_world.inverse(),
            area: shape.area(),
            n_samples
        }
    }
}

impl AreaLight for DiffuseAreaLight<'_> {
    fn emitted_radiance(&self, hit: SurfaceHit, w: Vec3f) -> Spectrum {
        if hit.n.dot(w) > 0.0 {
            self.emit
        } else {
            Spectrum::new(0.0)
        }
    }
}

impl Light for DiffuseAreaLight<'_> {
    fn flags(&self) -> LightFlags {
        LightFlags::Area
    }

    fn light_to_world(&self) -> &Transform {
        &self.l2w
    }

    fn world_to_light(&self) -> &Transform {
        &self.w2l
    }

    fn n_samples(&self) -> usize {
        self.n_samples
    }

    fn sample_incident_radiance(&self, reference: &SurfaceHit, u: Point2<f32>) -> LiSample {
        let p_shape = self.shape.sample_from_ref(reference, u);
        let wi = (p_shape.p - reference.p).normalize();
        let pdf = self.shape.pdf_from_ref(reference, wi);
        let vis = VisibilityTester {
            p0: *reference,
            p1: p_shape,
        };
        let radiance = self.emitted_radiance(p_shape, -wi);
        LiSample {
            radiance,
            wi,
            pdf,
            vis
        }
    }

    fn pdf_incident_radiance(&self, reference: &SurfaceHit, wi: Vector3<f32>) -> f32 {
        self.shape.pdf_from_ref(reference, wi)
    }
}