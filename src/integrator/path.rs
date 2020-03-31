use crate::integrator::{IntegratorRadiance, uniform_sample_one_light};
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;
use crate::{Float, RayDifferential, abs_dot};
use bumpalo::Bump;
use crate::material::TransportMode;
use crate::reflection::BxDFType;

pub struct PathIntegrator {
    max_depth: u16,
    rr_threshold: Float,
}

impl PathIntegrator {
    pub fn new(max_depth: u16, rr_threshold: f32) -> Self {
        PathIntegrator { max_depth, rr_threshold }
    }
}

impl IntegratorRadiance for PathIntegrator {
    fn preprocess(&mut self, _scene: &Scene, _sampler: &mut dyn Sampler) {
    }

    fn incident_radiance(
        &self,
        ray: &mut RayDifferential,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        arena: &Bump,
        depth: u16,
    ) -> Spectrum {
        let mut path_radiance = Spectrum::uniform(0.0);
        let mut throughput = Spectrum::uniform(1.0);
        let mut bounces = 0;
        let mut ray = ray;

        // was the last outgoing sampled path direction due to specular reflection?
        let mut specular_bounce = false;

        loop {
            let si = scene.intersect(&mut ray.ray);

            // possibly add emitted light at intersection
            if bounces == 0 || specular_bounce {
                if let Some(si) = &si {
                    path_radiance += throughput * si.emitted_radiance(-ray.ray.dir);
                } else {
                    path_radiance += throughput * scene.environment_emitted_radiance(ray);
                }
            }

            // Terminate path if ray escaped or max_depth was reached
            if si.is_none() || bounces >= self.max_depth {
                break;
            }

            let mut si = si.unwrap(); // TODO clean up control flow?
            if let Some(bsdf) = si.compute_scattering_functions(ray, arena, true, TransportMode::Radiance) {
                // Sample illumination from lights to find path contribution
                // But skip for perfectly specular BSDFs
                if bsdf.num_components(BxDFType::all() & !BxDFType::SPECULAR) > 0 {
                    let direct = throughput * uniform_sample_one_light(&si, &bsdf, scene, arena, sampler);
                    path_radiance += direct;
                }

                // Sample BSDF to get new path direction
                let wo = -ray.ray.dir;
                let bsdf_sample = bsdf.sample_f(wo, sampler.get_2d(), BxDFType::all());
                if let Some(bsdf_sample) = bsdf_sample.filter(|s| !s.f.is_black()) {
                    throughput *= bsdf_sample.f * abs_dot(bsdf_sample.wi, si.shading_n.0) / bsdf_sample.pdf;
                    specular_bounce = bsdf_sample.sampled_type.contains(BxDFType::SPECULAR);
                    *ray = si.hit.spawn_ray_with_dfferentials(bsdf_sample.wi, ray.diff);
                } else {
                    break;
                }
            } else {
                // Skip over null bsdf without incrementing bounces
                *ray = si.hit.spawn_ray_with_dfferentials(ray.ray.dir, ray.diff);
                continue;
            }

            // Possibly terminate the path with Russian roulette
            if throughput.max_component_value() < self.rr_threshold && bounces > 3 {
                let q = Float::max(0.05, 1.0 - throughput.max_component_value());
                if sampler.get_1d() < q {
                    break;
                } else {
                    throughput /= 1.0 - q;
                }
            }
            bounces += 1;
        }
        path_radiance
    }
}
