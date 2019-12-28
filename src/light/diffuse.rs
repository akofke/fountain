use crate::{Float, Transform, Vec3f};
use crate::spectrum::{Spectrum, RGBSpectrum};
use std::rc::Rc;
use crate::shapes::Shape;
use crate::light::{AreaLight, Light, LiSample, LightFlags, VisibilityTester, AreaLightBuilder};
use crate::interaction::SurfaceHit;
use cgmath::{Vector3, InnerSpace, Point2};
use std::sync::Arc;

#[derive(Clone)]
pub struct DiffuseAreaLightBuilder {
    pub emit: Spectrum,
    pub n_samples: usize
}

impl<S: Shape> AreaLightBuilder<S> for DiffuseAreaLightBuilder {
    type Target = DiffuseAreaLight<S>;

    fn create(self, shape: Arc<S>) -> Self::Target {
        let tf = shape.object_to_world().clone();
        DiffuseAreaLight::new(self.emit, shape, self.n_samples)
    }
}

pub struct DiffuseAreaLight<S: Shape> {
    emit: Spectrum,
    shape: Arc<S>,
    area: Float,
    n_samples: usize
}

impl<S: Shape> DiffuseAreaLight<S> {
    pub fn new(emit: Spectrum, shape: Arc<S>, n_samples: usize) -> Self {
        let area = shape.area();
        Self {
            emit,
            shape,
            area,
            n_samples
        }
    }
}

impl<S: Shape> AreaLight for DiffuseAreaLight<S> {
    fn emitted_radiance(&self, hit: SurfaceHit, w: Vec3f) -> Spectrum {
        if hit.n.dot(w) > 0.0 {
            self.emit
        } else {
            Spectrum::new(0.0)
        }
    }

    fn as_light(&self) -> &dyn Light {
        self
    }
}

impl<S: Shape> Light for DiffuseAreaLight<S> {
    fn flags(&self) -> LightFlags {
        LightFlags::Area
    }

    fn light_to_world(&self) -> &Transform {
        self.shape.object_to_world()
    }

    fn world_to_light(&self) -> &Transform {
        self.shape.world_to_object()
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