use crate::{Point2i, Point2f, Float};
use rand_xoshiro::Xoshiro256Plus;
use rand::{SeedableRng, Rng};
use crate::sampler::Sampler;

pub struct RandomSampler {
    samples_per_pixel: u64,
    rng: Xoshiro256Plus,
    current_pixel_sample_num: u64,
}

impl RandomSampler {
    pub fn new_with_seed(samples_per_pixel: u64, seed: u64) -> Self {
        Self {
            samples_per_pixel,
            rng: Xoshiro256Plus::seed_from_u64(seed),
            current_pixel_sample_num: 0
        }
    }
}

impl Sampler for RandomSampler {
    fn start_pixel(&mut self, pixel: Point2i) {
        self.current_pixel_sample_num = 0;
    }

    fn start_next_sample(&mut self) -> bool {
        self.current_pixel_sample_num += 1;
        self.current_pixel_sample_num <= self.samples_per_pixel
    }

    fn get_1d(&mut self) -> Float {
        self.rng.gen()
    }

    fn get_2d(&mut self) -> Point2f {
        Point2f::new(self.rng.gen(), self.rng.gen())
    }

    fn request_1d_array(&mut self, len: usize) {
        unimplemented!()
    }

    fn request_2d_array(&mut self, len: usize) {
        unimplemented!()
    }

    fn clone_with_seed(&self, seed: u64) -> Box<dyn Sampler> {
        // TODO: how to base off initial seed or do we need to?
        Box::new(Self::new_with_seed(self.samples_per_pixel, seed))
    }

    fn samples_per_pixel(&self) -> u64 {
        self.samples_per_pixel
    }
}
