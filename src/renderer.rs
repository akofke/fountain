use crate::camera::Camera;
use crate::sampler::Sampler;
use crate::scene::Scene;

pub trait Renderer {
    fn render(&mut self, scene: &Scene);
}

pub trait SamplerIntegrator {

}

pub struct SamplerRenderer<C: Camera, S: Sampler> {
    camera: C,
    sampler: S,
}

impl<C: Camera, S: Sampler> SamplerRenderer<C, S> {

}

impl<C: Camera, S: Sampler> Renderer for SamplerRenderer<C, S> {
    fn render(&mut self, scene: &Scene) {
        // preprocess


        unimplemented!()
    }
}