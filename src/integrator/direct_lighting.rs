use crate::integrator::IntegratorRadiance;
use crate::sampler::Sampler;
use bumpalo::Bump;
use crate::RayDifferential;
use crate::spectrum::{RGBSpectrum, Spectrum};
use crate::scene::Scene;
use crate::material::TransportMode;

pub enum LightStrategy {
    UniformSampleAll, UniformSampleOne
}

pub struct DirectLightingIntegrator {
    strategy: LightStrategy,
    max_depth: u16,
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
                for &n_samples in &self.n_light_samples {
                    sampler.request_2d_array(n_samples);
                    sampler.request_2d_array(n_samples);
                }
            }
        }
    }

    fn incident_radiance(&self, ray: &mut RayDifferential, scene: &Scene, sampler: &mut dyn Sampler, arena: &Bump, depth: u16) -> Spectrum {
        let mut radiance: Spectrum = Spectrum::new(0.0);

        match scene.intersect(&mut ray.ray) {
            None => {
                // get radiance of escaping ray
//                background(ray.ray.dir)
                Spectrum::new(0.0)
            },

            Some(mut intersect) => {
                let n = intersect.shading_n;
                let wo = intersect.wo;

                let bsdf = intersect.compute_scattering_functions(
                    ray,
                    arena,
                    false,
                    TransportMode::Radiance
                );

                if let Some(bsdf) = bsdf {

                    if depth + 1 < self.max_depth {
                        radiance += self.specular_reflect(ray, &intersect, &bsdf, scene, sampler, arena, depth);
                        radiance += self.specular_transmit(ray, &intersect, &bsdf, scene, sampler, arena, depth);
                    }
                } else {
                    unimplemented!()
                }

                radiance
            }

        }
    }
}