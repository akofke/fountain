use crate::{Float, Point2f, Point2i};

pub mod random;

pub trait Sampler {
    fn start_pixel(&mut self, pixel: Point2i);

    fn start_next_sample(&mut self) -> bool;

    fn get_1d(&mut self) -> Float;

    fn get_2d(&mut self) -> Point2f;

    fn request_1d_array(&mut self, len: usize);

    fn request_2d_array(&mut self, len: usize);

    fn round_count(&self, n: usize) -> usize { n }


}
