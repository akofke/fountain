use cgmath::EuclideanSpace;

use crate::{Float, Point2f, Point2i};
use crate::camera::CameraSample;

pub mod random;

pub trait Sampler: Sync + Send {
    fn start_pixel(&mut self, pixel: Point2i);

    fn start_next_sample(&mut self) -> bool;

    fn get_1d(&mut self) -> Float;

    fn get_2d(&mut self) -> Point2f;

    fn request_1d_array(&mut self, len: usize);

    fn request_2d_array(&mut self, len: usize);

    fn get_1d_array(&mut self, len: usize) -> &[Float];

    fn get_2d_array(&mut self, len: usize) -> &[Point2f];

    fn round_count(&self, n: usize) -> usize { n }

    fn clone_with_seed(&self, seed: u64) -> Box<dyn Sampler>;

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

pub struct SamplerState {
    samples_per_pixel: usize,
    current_pixel: Point2i,
    current_pixel_sample_num: usize,

    sample_array_1d: Vec<Vec<Float>>,
    sample_array_2d: Vec<Vec<Point2f>>,
    samples_1d_array_sizes: Vec<usize>,
    samples_2d_array_sizes: Vec<usize>,
    array_1d_offset: usize,
    array_2d_offset: usize,

}

impl SamplerState {
    pub fn new(samples_per_pixel: usize) -> Self {
        Self {
            samples_per_pixel,
            current_pixel: Point2i::new(0, 0),
            current_pixel_sample_num: 0,
            sample_array_1d: vec![],
            sample_array_2d: vec![],
            samples_1d_array_sizes: vec![],
            samples_2d_array_sizes: vec![],
            array_1d_offset: 0,
            array_2d_offset: 0
        }
    }

    pub fn start_pixel(&mut self, p: Point2i) {
        self.current_pixel = p;
        self.current_pixel_sample_num = 0;
        self.array_1d_offset = 0;
        self.array_2d_offset = 0;
    }

    pub fn start_next_sample(&mut self) -> bool {
        self.array_1d_offset = 0;
        self.array_2d_offset = 0;
        self.current_pixel_sample_num += 1;
        self.current_pixel_sample_num < self.samples_per_pixel
    }

    pub fn request_1d_array(&mut self, len: usize) {
        self.samples_1d_array_sizes.push(len);
        self.sample_array_1d.push(Vec::with_capacity(len * self.samples_per_pixel as usize))
    }

    pub fn request_2d_array(&mut self, len: usize) {
        self.samples_2d_array_sizes.push(len);
        self.sample_array_2d.push(Vec::with_capacity(len * self.samples_per_pixel as usize))
    }

    pub fn get_1d_array(&mut self, len: usize) -> &[Float] {
        let range = (self.current_pixel_sample_num * len .. (self.current_pixel_sample_num + 1) * len);
        let array = &self.sample_array_1d[self.array_1d_offset][range];
        self.array_1d_offset += 1;
        array
    }

    pub fn get_2d_array(&mut self, len: usize) -> &[Point2f] {
        let range = (self.current_pixel_sample_num * len .. (self.current_pixel_sample_num + 1) * len);
        let array = &self.sample_array_2d[self.array_2d_offset][range];
        self.array_2d_offset += 1;
        array
    }
}
