use crate::{Float, Point2i, Bounds2i, Bounds2f, Point2f};
use crate::filter::Filter;

const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Default)]
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
        resolution: Point2i,
        crop_window: Bounds2f,
        filter: F,
        diagonal: Float
    ) -> Self {
        let low_x = (resolution.x as Float * crop_window.min.x).ceil() as i32;
        let low_y = (resolution.y as Float * crop_window.min.y).ceil() as i32;
        let high_x = (resolution.x as Float * crop_window.max.x).ceil() as i32;
        let high_y = (resolution.y as Float * crop_window.max.y).ceil() as i32;

        let cropped_pixel_bounds = Bounds2i::with_bounds(
            Point2i::new(low_x, low_y),
            Point2i::new(high_x, high_y)
        );

        let mut pixels = Vec::with_capacity(cropped_pixel_bounds.area() as usize);
        pixels.iter_mut().for_each(|p| *p = Pixel::default());

        let mut filter_table = [[0.0f32; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for (x, row) in filter_table.iter_mut().enumerate() {
            for (y, val) in row.iter_mut().enumerate() {
                let p = Point2f::new(
                    (x as Float + 0.5) * filter.radius().x / FILTER_TABLE_WIDTH as Float,
                    (y as Float + 0.5) * filter.radius().y / FILTER_TABLE_WIDTH as Float
                );

                *val = filter.evaluate(&p);
            }
        }

        Self {
            full_resolution: resolution,
            cropped_pixel_bounds,
            diagonal,
            filter,
            pixels,
            filter_table,
        }
    }
}

