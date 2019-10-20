use crate::integrator::IntegratorRadiance;
use crate::sampler::Sampler;
use bumpalo::Bump;
use crate::RayDifferential;
use crate::spectrum::{RGBSpectrum, Spectrum};
use crate::scene::Scene;

pub enum LightStrategy {
    UniformSampleAll, UniformSampleOne
}

pub struct DirectLightingIntegrator {
    strategy: LightStrategy,
    max_depth: u32,
    n_light_samples: Vec<usize>,
}

impl DirectLightingIntegrator {

}

impl IntegratorRadiance for DirectLightingIntegrator {
    fn preprocess(&mut self, scene: &Scene, sampler: &mut dyn Sampler) {
        if let LightStrategy::UniformSampleAll = self.strategy {

            // Store the number of samples to be used for each light.
            // TODO: give each light an id? Currently just relies on consistent iteration order.
            self.n_light_samples = scene.lights.iter()
                .map(|light| sampler.round_count(light.n_samples()))
                .collect();

            for _ in 0..self.max_depth {
                for n_samples in self.n_light_samples {
                    sampler.request_2d_array(n_samples);
                    sampler.request_2d_array(n_samples);
                }
            }
        }
    }

    fn incident_radiance(&self, ray: &mut RayDifferential, scene: &Scene, sampler: &mut dyn Sampler, arena: &Bump, depth: u16) -> Spectrum {
        unimplemented!()
    }
}