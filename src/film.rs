use crate::{Float, Point2i, Bounds2i, Bounds2f, Point2f, Vec2f, Vec2i, ComponentWiseExt};
use crate::filter::Filter;
use crate::spectrum::{Spectrum, RGBSpectrum};
use cgmath::vec2;
use smallvec::SmallVec;

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

struct FilmTilePixel {
    contrib_sum: Spectrum<RGBSpectrum>,
    filter_weight_sum: Float,
}

pub struct FilmTile<'a, F: Filter> {
    pixel_bounds: Bounds2i,
    filter_radius: Vec2f,
    inv_filter_radius: Vec2f,
    film: &'a Film<F>,
    pixels: Vec<FilmTilePixel>
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
        for (y, row) in filter_table.iter_mut().enumerate() {
            for (x, val) in row.iter_mut().enumerate() {
                let p = Point2f::new(
                    (x as Float + 0.5) * filter.radius().0.x / FILTER_TABLE_WIDTH as Float,
                    (y as Float + 0.5) * filter.radius().0.y / FILTER_TABLE_WIDTH as Float
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

    /// The range of pixel values that must be sampled,
    /// this is larger than the size of the image to allow pixels
    /// at the edge to have an equal number of samples.
    pub fn sample_bounds(&self) -> Bounds2i {
        let low_x = (self.cropped_pixel_bounds.min.x as Float + 0.5 - self.filter.radius().0.x).floor() as i32;
        let low_y = (self.cropped_pixel_bounds.min.y as Float + 0.5 - self.filter.radius().0.y).floor() as i32;
        let high_x = (self.cropped_pixel_bounds.max.x as Float - 0.5 + self.filter.radius().0.x).ceil() as i32;
        let high_y = (self.cropped_pixel_bounds.max.y as Float - 0.5 + self.filter.radius().0.y).ceil() as i32;

        Bounds2i::with_bounds(Point2i::new(low_x, low_y), Point2i::new(high_x, high_y))
    }

    pub fn get_film_tile(&self, sample_bounds: Bounds2i) -> FilmTile<F> {
        let half_pixel = Vec2f::new(0.5, 0.5);
        let p0x = (sample_bounds.min.x as Float - 0.5 - self.filter.radius().0.x).ceil() as i32;
        let p0y = (sample_bounds.min.y as Float - 0.5 - self.filter.radius().0.y).ceil() as i32;

        let p1x = (sample_bounds.max.x as Float - 0.5 + self.filter.radius().0.x + 1.0).ceil() as i32;
        let p1y = (sample_bounds.max.y as Float - 0.5 - self.filter.radius().0.y + 1.0).ceil() as i32;

        let p0 = Point2i::new(p0x, p0y);
        let p1 = Point2i::new(p1x, p1y);

        let tile_pixel_bounds = Bounds2i::with_bounds(p0, p1).intersection(&self.cropped_pixel_bounds);

        FilmTile {
            pixel_bounds: tile_pixel_bounds,
            filter_radius: self.filter.radius().0,
            inv_filter_radius: self.filter.radius().1,
            film: &self,
            pixels: Vec::with_capacity(tile_pixel_bounds.area().max(0) as usize)
        }
    }
}

impl<'a, F: Filter> FilmTile<'a, F> {

    pub fn add_sample(&mut self, p_film: Point2f, radiance: Spectrum, sample_weight: Float) {
        let p_film_discrete = p_film - vec2(0.5, 0.5);
        let p0: Point2i = (p_film_discrete - self.filter_radius).map(|v| v.ceil()).cast().unwrap();
        let p1: Point2i = (p_film_discrete + self.filter_radius).map(|v| v.floor()).cast::<i32>().unwrap() + Vec2i::new(1, 1);

        let p0 = p0.max(self.pixel_bounds.min);
        let p1 = p1.min( self.pixel_bounds.max);

        let mut filter_indices_x = SmallVec::<[usize; 64]>::from_elem(0, (p1.x - p0.x) as usize);
        for x in p0.x..p1.x {
            let filt_x = ((x as Float - p_film_discrete.x) * self.inv_filter_radius.x * FILTER_TABLE_WIDTH as Float).abs();

            let i = (x - p0.x) as usize;
            filter_indices_x[i] = (filt_x.floor() as usize).min(FILTER_TABLE_WIDTH - 1);
        }

        let mut filter_indices_y = SmallVec::<[usize; 64]>::from_elem(0, (p1.y - p0.y) as usize);
        for y in p0.y..p1.y {
            let filt_y = ((y as Float - p_film_discrete.y) * self.inv_filter_radius.y * FILTER_TABLE_WIDTH as Float).abs();

            let i = (y - p0.y) as usize;
            filter_indices_y[i] = (filt_y.floor() as usize).min(FILTER_TABLE_WIDTH - 1);
        }

        for y in p0.y..p1.y {
            for x in p0.x..p1.x {
                let y_idx = filter_indices_y[(y - p0.y) as usize];
                let x_idx = filter_indices_x[(x - p0.x) as usize];

                let filter_weight = self.film.filter_table[y_idx][x_idx];
                let idx = self.get_pixel_idx(Point2i::new(x, y));
                let pixel = &mut self.pixels[idx];
                pixel.contrib_sum += radiance * sample_weight * filter_weight;
                pixel.filter_weight_sum += filter_weight;
            }
        }
    }

    /// Gets the FilmTilePixel of a FilmTile given pixel coordinates with respect to the overall
    /// image
    fn get_pixel_idx(&self, p: Point2i) -> usize {
        let width = self.pixel_bounds.max.x - self.pixel_bounds.min.x;
        let idx = (p.y - self.pixel_bounds.min.y) * width + (p.x - self.pixel_bounds.min.x);
        idx as usize
    }
}


