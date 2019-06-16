use crate::{Float, Point2i, Bounds2i};
use crate::filter::Filter;

const FILTER_TABLE_WIDTH: usize = 16;

pub struct Pixel {
    pub rgb: [Float; 3],
    pub filter_weight_sum: Float,
}

pub struct Film<F: Filter> {
    pub full_resolution: Point2i,
    pub cropped_pixel_bounds: Bounds2i,
    pub diagonal: Float,
    pub filter: F,
    pub pixels: Vec<Pixel>,
    filter_table: [[Float; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH],
}

impl<F: Filter> Film<F> {
    pub fn new(

    ) -> Self {

    }
}

