use crate::camera::Camera;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::RayDifferential;
use bumpalo::Bump;

pub trait Integrator {
    fn render(&mut self, scene: &Scene);
}

pub struct SamplerIntegrator<R: IntegratorRadiance> {
    camera: Box<dyn Camera>,
    sampler: Box<dyn Sampler>,
    radiance: R,
}

pub trait IntegratorRadiance {
    fn preprocess(&mut self, scene: &Scene, sampler: &dyn Sampler);

    fn radiance(&self, ray: &RayDifferential, scene: &Scene, sampler: &dyn Sampler, arena: &Bump, depth: u16);
}


impl<R: IntegratorRadiance> Integrator for SamplerIntegrator<R> {
    fn render(&mut self, scene: &Scene) {
        // preprocess


        unimplemented!()
    }
}

impl<R: IntegratorRadiance> SamplerIntegrator<R> {

}
