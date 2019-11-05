use cgmath::EuclideanSpace;

use crate::{Float, Point2f, Point2i};
use crate::camera::CameraSample;
use std::cell::Cell;
use std::sync::Arc;
use ndarray::prelude::*;

pub mod random;

#[derive(Copy, Clone)]
pub struct SampleArrayId {
    /// The index of the sample group.
    idx: usize,

    /// The length of the requested sample array
    len: usize,
}

pub trait Sampler: Send {
    fn start_pixel(&mut self, pixel: Point2i);

    fn start_next_sample(&mut self) -> bool;

    fn get_1d(&mut self) -> Float;

    fn get_2d(&mut self) -> Point2f;

    fn request_1d_array(&mut self, len: usize) -> SampleArrayId;

    fn request_2d_array(&mut self, len: usize) -> SampleArrayId;

    // TODO: find better way to keep &mut self api but still return reference (i.e. borrow regions)
    fn get_1d_array(&self, id: SampleArrayId) -> &[Float];

    fn get_2d_array(&self, id: SampleArrayId) -> &[Point2f];

    fn round_count(&self, n: usize) -> usize { n }

    fn clone_with_seed(&self, seed: u64) -> Self where Self: Sized;

    fn samples_per_pixel(&self) -> usize;

    fn get_camera_sample(&mut self, p_raster: Point2i) -> CameraSample {
        let p_film = p_raster.cast::<Float>().unwrap() + self.get_2d().to_vec();

        CameraSample {
            p_film,
            p_lens: self.get_2d(),
            time: self.get_1d(),
        }
    }

    fn set_sample_number(&mut self, sample_num: u64) -> bool;
}

#[derive(Clone)]
pub struct SamplerState {
    samples_per_pixel: usize,
    current_pixel: Point2i,
    current_pixel_sample_num: usize,

    sample_array_1d: Vec<Array2<Float>>,
    sample_array_2d: Vec<Array2<Point2f>>,

    // Store a vector of grouped samples. For each group, store an array of samples of the
    // requested size for
//    sample_array_1d: Vec<Vec<Float>>,
//    sample_array_2d: Vec<Vec<Point2f>>,
//    samples_1d_array_sizes: Vec<usize>,
//    samples_2d_array_sizes: Vec<usize>,
//    array_1d_offset: Cell<usize>,
//    array_2d_offset: Cell<usize>,

}

impl SamplerState {
    pub fn new(samples_per_pixel: usize) -> Self {
        Self {
            samples_per_pixel,
            current_pixel: Point2i::new(0, 0),
            current_pixel_sample_num: 0,
            sample_array_1d: vec![],
            sample_array_2d: vec![],
        }
    }

    pub fn start_pixel(&mut self, p: Point2i) {
        self.current_pixel = p;
        self.current_pixel_sample_num = 0;
//        self.array_1d_offset = 0.into();
//        self.array_2d_offset = 0.into();
    }

    pub fn start_next_sample(&mut self) -> bool {
//        self.array_1d_offset = 0.into();
//        self.array_2d_offset = 0.into();
        self.current_pixel_sample_num += 1;
        self.current_pixel_sample_num < self.samples_per_pixel
    }

    pub fn request_1d_array(&mut self, len: usize) -> SampleArrayId {
        let id = SampleArrayId {
            idx: self.sample_array_1d.len(),
            len
        };
        self.sample_array_1d.push(Array2::zeros((self.samples_per_pixel, len)));
        id
//        self.sample_array_1d.push(vec!(0.0; len * self.samples_per_pixel as usize))
    }

    pub fn request_2d_array(&mut self, len: usize) -> SampleArrayId {
        let id = SampleArrayId {
            idx: self.sample_array_2d.len(),
            len
        };
        self.sample_array_2d.push(Array2::from_elem((self.samples_per_pixel, len), Point2f::origin()));
        id
    }

    pub fn get_1d_array(&self, id: SampleArrayId) -> &[Float] {
        unimplemented!()
//        let sample_array = &self.sample_array_1d[id.idx];
//        let arr = sample_array.row(self.current_pixel_sample_num);
//        arr.as_slice().unwrap()

//        let range = (self.current_pixel_sample_num * len .. (self.current_pixel_sample_num + 1) * len);
//        let array = &self.sample_array_1d[self.array_1d_offset.get()][range];
//        self.array_1d_offset.replace(self.array_1d_offset.get() + 1);
//        array
    }

    pub fn get_2d_array(&self, id: SampleArrayId) -> &[Point2f] {
        unimplemented!()
//        let sample_array = &self.sample_array_2d[id.idx];
//        let arr = sample_array.row(self.current_pixel_sample_num);
//        arr.as_slice().unwrap()

//        let range = (self.current_pixel_sample_num * len .. (self.current_pixel_sample_num + 1) * len);
//        let array = &self.sample_array_2d[self.array_2d_offset.get()][range];
//        self.array_2d_offset.replace(self.array_2d_offset.get() + 1);
//        array
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::random::RandomSampler;

    #[test]
    fn test_get_sample_arrays() {
        let mut sampler = RandomSampler::new_with_seed(2, 0);

        sampler.request_1d_array(10);
        sampler.request_1d_array(7);

        sampler.start_pixel((0, 0).into());

        let arr = sampler.get_1d_array(10);
        assert_eq!(arr.len(), 10);
        assert!(arr.iter().any(|&x| x > 0.0));

        let arr = sampler.get_1d_array(7);
        assert_eq!(arr.len(), 7);
        assert!(arr.iter().any(|&x| x > 0.0));

        assert!(sampler.start_next_sample());

        let arr = sampler.get_1d_array(10);
        assert_eq!(arr.len(), 10);
        assert!(arr.iter().any(|&x| x > 0.0));

        let arr = sampler.get_1d_array(7);
        assert_eq!(arr.len(), 7);
        assert!(arr.iter().any(|&x| x > 0.0));

        assert!(!sampler.start_next_sample());

        sampler.start_pixel((1, 1).into());

        let arr = sampler.get_1d_array(10);
        assert_eq!(arr.len(), 10);
        assert!(arr.iter().any(|&x| x > 0.0));

        let arr = sampler.get_1d_array(7);
        assert_eq!(arr.len(), 7);
        assert!(arr.iter().any(|&x| x > 0.0));

        assert!(sampler.start_next_sample());

        let arr = sampler.get_1d_array(10);
        assert_eq!(arr.len(), 10);
        assert!(arr.iter().any(|&x| x > 0.0));

        let arr = sampler.get_1d_array(7);
        assert_eq!(arr.len(), 7);
        assert!(arr.iter().any(|&x| x > 0.0));

        assert!(!sampler.start_next_sample());
    }
}
