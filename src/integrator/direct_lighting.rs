use crate::integrator::{IntegratorRadiance, uniform_sample_one_light};
use crate::sampler::Sampler;
use bumpalo::Bump;
use crate::{RayDifferential, SurfaceInteraction, Point2f, abs_dot, Float};
use crate::spectrum::{Spectrum};
use crate::scene::Scene;
use crate::material::TransportMode;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::BxDFType;
use crate::light::Light;
use crate::sampling::power_heuristic;

pub enum LightStrategy {

    /// Loops over all of the lights and takes a number of samples from each based on
    /// `Light::n_samples` and sums the result. This applies the Monte Carlo technique of splitting.
    UniformSampleAll,

    /// Takes a single sample from one of the lights, chosen at random.
    UniformSampleOne
}

pub struct DirectLightingIntegrator {
    pub strategy: LightStrategy,
    pub max_depth: u16,
    pub n_light_samples: Vec<usize>,
//    pub light_sample_ids:
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
        let mut radiance: Spectrum = Spectrum::uniform(0.0);

        match scene.intersect(&mut ray.ray) {
            None => {
                scene.environment_emitted_radiance(ray)
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

                    // Add emitted light if ray hit an area light source.
                    radiance += intersect.emitted_radiance(intersect.wo);

                    radiance += match self.strategy {
                        LightStrategy::UniformSampleAll => {
                            uniform_sample_all_lights(
                                &intersect,
                                &bsdf,
                                scene,
                                arena,
                                sampler,
                                &self.n_light_samples
                            )
                        },
                        LightStrategy::UniformSampleOne => {
                            uniform_sample_one_light(
                                &intersect,
                                &bsdf,
                                scene,
                                arena,
                                sampler,
                            )
                        }
                    };

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

fn uniform_sample_all_lights(
    intersect: &SurfaceInteraction,
    bsdf: &Bsdf,
    scene: &Scene,
    arena: &Bump,
    sampler: &mut dyn Sampler,
    n_light_samples: &[usize],
) -> Spectrum {
    unimplemented!()
//    scene.lights.iter().zip(n_light_samples).map(|(light, &n_samples)| {
//        // TODO: sampler return optional arrays
//        let u_light_array = sampler.get_2d_array(n_samples);
//        let u_scattering_array = sampler.get_2d_array(n_samples);
//
//        u_light_array.iter().zip(u_scattering_array)
//            .map(|(&u_light, &u_scattering)| {
//                estimate_direct(
//                    bsdf,
//                    intersect,
//                    u_scattering,
//                    *light,
//                    u_light,
//                    scene,
//                    arena,
////                    sampler, // TODO: ??? would be needed for volumes
//                )
//            }).sum::<Spectrum>() / (n_samples as Float)
//    }).sum()
}
