use crate::{Float, Point2f, Point2i, Vec2f};
use crate::camera::CameraSample;
use cgmath::EuclideanSpace;

pub mod random;

pub trait Sampler: Sync + Send {
    fn start_pixel(&mut self, pixel: Point2i);

    fn start_next_sample(&mut self) -> bool;

    fn get_1d(&mut self) -> Float;

    fn get_2d(&mut self) -> Point2f;

    fn request_1d_array(&mut self, len: usize);

    fn request_2d_array(&mut self, len: usize);

    fn round_count(&self, n: usize) -> usize { n }

    fn clone_with_seed(&self, seed: u64) -> Box<dyn Sampler>;

    fn samples_per_pixel(&self) -> u64;

    fn get_camera_sample(&mut self, p_raster: Point2i) -> CameraSample {
        let p_film = p_raster.cast::<Float>().unwrap() + self.get_2d().to_vec();

        CameraSample {
            p_film,
            p_lens: self.get_2d(),
            time: self.get_1d(),
        }
    }
}
