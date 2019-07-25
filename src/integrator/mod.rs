use crate::camera::Camera;
use crate::film::Film;
use crate::filter::BoxFilter;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::{RGBSpectrum, Spectrum};
use crate::{Bounds2i, Float, RayDifferential, SurfaceInteraction};
use bumpalo::Bump;
use rayon::prelude::*;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::BxDFType;

pub mod whitted;

pub trait Integrator {
    fn render(&mut self, scene: &Scene, film: &Film<BoxFilter>);
}

pub struct SamplerIntegrator<R: IntegratorRadiance> {
    camera: Box<dyn Camera>,
    sampler: Box<dyn Sampler>,
    radiance: R,
}

pub trait IntegratorRadiance: Sync {
    fn preprocess(&mut self, scene: &Scene, sampler: &dyn Sampler);

    fn incident_radiance(
        &self,
        ray: &mut RayDifferential,
        scene: &Scene,
        sampler: &dyn Sampler,
        arena: &Bump,
        depth: u16,
    ) -> Spectrum;
}

impl<R: IntegratorRadiance> Integrator for SamplerIntegrator<R> {
    fn render(&mut self, scene: &Scene, film: &Film<BoxFilter>) {
        // preprocess
        let sample_bounds = film.sample_bounds();
        sample_bounds.iter_tiles(16).par_bridge().for_each(|tile| {
            let mut arena = Bump::new();

            let tile_id = Self::tile_id(tile, sample_bounds);
            let mut tile_sampler = self.sampler.clone_with_seed(tile_id);

            let mut film_tile = film.get_film_tile(tile);

            for pixel in tile.iter_points() {
                tile_sampler.start_pixel(pixel.into());

                while tile_sampler.start_next_sample() {
                    let camera_sample = tile_sampler.get_camera_sample(pixel.into());

                    let (ray_weight, mut ray_differential) =
                        self.camera.generate_ray_differential(camera_sample);

                    ray_differential.scale_differentials(
                        1.0 / (tile_sampler.samples_per_pixel() as Float).sqrt(),
                    );

                    let mut radiance = Spectrum::<RGBSpectrum>::new(0.0);

                    if ray_weight > 0.0 {
                        radiance = self.radiance.incident_radiance(
                            &mut ray_differential,
                            scene,
                            tile_sampler.as_ref(),
                            &arena,
                            0,
                        );
                        // TODO: check value
                    }

                    film.add_sample_to_tile(
                        &mut film_tile,
                        camera_sample.p_film,
                        radiance,
                        ray_weight,
                    );

                    arena.reset();
                }
            }

            film.merge_film_tile(film_tile);
        });

        unimplemented!()
    }
}

impl<R: IntegratorRadiance> SamplerIntegrator<R> {
    fn tile_id(tile: Bounds2i, sample_bounds: Bounds2i) -> u64 {
        let n_cols = sample_bounds.max.x;
        (tile.min.y * n_cols + tile.min.x) as u64
    }

    pub fn specular_reflect(
        ray: &mut RayDifferential,
        intersect: &SurfaceInteraction,
        bsdf: &Bsdf,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        arena: &Bump,
        depth: u16,
    ) -> Spectrum {
        let wo = intersect.wo;
        let bxdf_type = BxDFType::REFLECTION | BxDFType::SPECULAR;

        let f = bsdf.sample_f(wo, sampler.get_2d(), bxdf_type);
        unimplemented!()
    }
}
