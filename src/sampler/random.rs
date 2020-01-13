use crate::{Point2i, Point2f, Float};
use rand_xoshiro::Xoshiro256Plus;
use rand::{SeedableRng, Rng};
use crate::sampler::{Sampler, SamplerState, SampleArrayId};
use cgmath::Point2;
use crate::spectrum::xyz_to_rgb;
use std::sync::Arc;

pub struct RandomSampler {
    rng: Xoshiro256Plus,
    state: SamplerState,
}

impl RandomSampler {
    pub fn new_with_seed(samples_per_pixel: usize, seed: u64) -> Self {
        Self {
            rng: Xoshiro256Plus::seed_from_u64(seed),
            state: SamplerState::new(samples_per_pixel),
        }
    }
}

impl Sampler for RandomSampler {
    fn start_pixel(&mut self, pixel: Point2i) {
        self.state.start_pixel(pixel);
        let rng = &mut self.rng;
//        self.state.sample_array_1d.iter_mut().flatten().for_each(|x| {
//            *x = rng.gen();
//        });
//
//        self.state.sample_array_2d.iter_mut().flatten().for_each(|p| {
//            *p = Point2f::new(rng.gen(), rng.gen());
//        });
    }

    fn start_next_sample(&mut self) -> bool {
        self.state.start_next_sample()
    }

    fn get_1d(&mut self) -> Float {
        self.rng.gen()
    }

    fn get_2d(&mut self) -> Point2f {
        Point2f::new(self.rng.gen(), self.rng.gen())
    }

    fn request_1d_array(&mut self, len: usize) -> SampleArrayId {
        self.state.request_1d_array(len)
    }

    fn request_2d_array(&mut self, len: usize) -> SampleArrayId {
        self.state.request_2d_array(len)
    }

    fn get_1d_array(&self, id: SampleArrayId) -> &[Float] {
        self.state.get_1d_array(id)
    }

    fn get_2d_array(&self, id: SampleArrayId) -> &[Point2f] {
        self.state.get_2d_array(id)
    }

    fn clone_with_seed(&self, seed: u64) -> Self where Self: Sized {
        // TODO: how to base off initial seed or do we need to?
        Self {
            rng: Xoshiro256Plus::seed_from_u64(seed),
            state: self.state.clone(),
        }
    }

    fn samples_per_pixel(&self) -> usize {
        self.state.samples_per_pixel
    }

    fn set_sample_number(&mut self, sample_num: u64) -> bool {
        unimplemented!()
    }
}
