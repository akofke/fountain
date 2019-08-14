use crate::{Float, Point2i, Bounds2i, Bounds2f, Point2f, Vec2f, Vec2i, ComponentWiseExt};
use crate::filter::Filter;
use crate::spectrum::{Spectrum, RGBSpectrum, CoefficientSpectrum, xyz_to_rgb};
use cgmath::vec2;
use smallvec::SmallVec;
use parking_lot::Mutex;
use image::{ImageBuffer, Rgb};
use arrayvec::ArrayVec;

const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Pixel {
    pub xyz: [Float; 3],
    pub filter_weight_sum: Float,
}

#[derive(Debug)]
pub struct Film<F: Filter> {
    pub full_resolution: Point2i,
    pub cropped_pixel_bounds: Bounds2i,
    pub diagonal: Float,
    pub filter: F,
    pub pixels: Mutex<Vec<Pixel>>,
    filter_table: [[Float; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH],
}

#[derive(Debug, Clone, Copy, Default)]
struct FilmTilePixel {
    contrib_sum: Spectrum<RGBSpectrum>,
    filter_weight_sum: Float,
}

#[derive(Debug)]
pub struct FilmTile {
    pixel_bounds: Bounds2i,
    filter_radius: Vec2f,
    inv_filter_radius: Vec2f,
    pixels: Vec<FilmTilePixel>,
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

        let pixels = vec![Default::default(); cropped_pixel_bounds.area() as usize];

        let mut filter_table = [[0.0f32; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for (y, row) in filter_table.iter_mut().enumerate() {
            for (x, val) in row.iter_mut().enumerate() {
                let p = Point2f::new(
                    (x as Float + 0.5) * filter.radius().0.x / FILTER_TABLE_WIDTH as Float,
                    (y as Float + 0.5) * filter.radius().0.y / FILTER_TABLE_WIDTH as Float
                );

                *val = filter.evaluate(p);
            }
        }

        Self {
            full_resolution: resolution,
            cropped_pixel_bounds,
            diagonal,
            filter,
            pixels: Mutex::new(pixels),
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

    pub fn get_film_tile(&self, sample_bounds: Bounds2i) -> FilmTile {
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
            pixels: vec![Default::default(); tile_pixel_bounds.area().max(0) as usize],
        }
    }

    pub fn get_pixel_idx(&self, p: Point2i) -> usize {
        let width = self.cropped_pixel_bounds.max.x - self.cropped_pixel_bounds.min.x;
        let offset = (p.x - self.cropped_pixel_bounds.min.x) + (p.y - self.cropped_pixel_bounds.min.y) * width;
        offset as usize
    }

    pub fn merge_film_tile(&self, tile: FilmTile) {
        let mut pixels = self.pixels.lock();
        for pixel in tile.pixel_bounds.iter_points() {
            let film_tile_pixel = &tile.pixels[tile.get_pixel_idx(pixel.into())];
            let merge_pixel = &mut pixels[self.get_pixel_idx(pixel.into())];
            let xyz = film_tile_pixel.contrib_sum.to_xyz();
            for i in 0..3 {
                merge_pixel.xyz[i] += xyz[i];
            }
            merge_pixel.filter_weight_sum += film_tile_pixel.filter_weight_sum;
        }
    }

    // this satisfies the borrow checker when borrowing mutably to merge film tile, since the tile doesn't need to hold a reference
    // to the filter table and instead it is passed every time.
    pub fn add_sample_to_tile(&self, tile: &mut FilmTile, p_film: Point2f, radiance: Spectrum, sample_weight: Float) {
        let p_film_discrete = p_film - vec2(0.5, 0.5);
        let p0: Point2i = (p_film_discrete - tile.filter_radius).map(|v| v.ceil()).cast().unwrap();
        let p1: Point2i = (p_film_discrete + tile.filter_radius).map(|v| v.floor()).cast::<i32>().unwrap() + Vec2i::new(1, 1);

        let p0 = p0.max(tile.pixel_bounds.min);
        let p1 = p1.min(tile.pixel_bounds.max);

        let mut filter_indices_x = SmallVec::<[usize; 64]>::from_elem(0, (p1.x - p0.x) as usize);
        for x in p0.x..p1.x {
            let filt_x = ((x as Float - p_film_discrete.x) * tile.inv_filter_radius.x * FILTER_TABLE_WIDTH as Float).abs();

            let i = (x - p0.x) as usize;
            filter_indices_x[i] = (filt_x.floor() as usize).min(FILTER_TABLE_WIDTH - 1);
        }

        let mut filter_indices_y = SmallVec::<[usize; 64]>::from_elem(0, (p1.y - p0.y) as usize);
        for y in p0.y..p1.y {
            let filt_y = ((y as Float - p_film_discrete.y) * tile.inv_filter_radius.y * FILTER_TABLE_WIDTH as Float).abs();

            let i = (y - p0.y) as usize;
            filter_indices_y[i] = (filt_y.floor() as usize).min(FILTER_TABLE_WIDTH - 1);
        }

        for y in p0.y..p1.y {
            for x in p0.x..p1.x {
                let y_idx = filter_indices_y[(y - p0.y) as usize];
                let x_idx = filter_indices_x[(x - p0.x) as usize];

                let filter_weight = self.filter_table[y_idx][x_idx];
                let idx = tile.get_pixel_idx(Point2i::new(x, y));
                let pixel = &mut tile.pixels[idx];
                pixel.contrib_sum += radiance * sample_weight * filter_weight;
                pixel.filter_weight_sum += filter_weight;
            }
        }
    }

    pub fn into_image_buffer(self) -> ImageBuffer<Rgb<f32>, Vec<f32>> {
        let pixels = self.pixels.into_inner();
        let rgb_flat_buffer: Vec<Float> = pixels.into_iter().flat_map(|pixel| {
            let mut rgb = xyz_to_rgb(pixel.xyz);
            if pixel.filter_weight_sum != 0.0 {
                let inv_wt = 1.0 / pixel.filter_weight_sum;
                for val in &mut rgb {
                    *val = Float::max(0.0, *val * inv_wt);
                }
            }
            ArrayVec::from(rgb)
        }).collect();

        let (width, height) = self.cropped_pixel_bounds.dimensions();
        ImageBuffer::from_vec(
            width as u32,
            height as u32,
            rgb_flat_buffer
        ).expect("Invalid dimensions when creating image buffer")
    }
}

impl FilmTile {


    /// Gets the FilmTilePixel of a FilmTile given pixel coordinates with respect to the overall
    /// image
    fn get_pixel_idx(&self, p: Point2i) -> usize {
        let width = self.pixel_bounds.max.x - self.pixel_bounds.min.x;
        let idx = (p.y - self.pixel_bounds.min.y) * width + (p.x - self.pixel_bounds.min.x);
        idx as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::BoxFilter;
    use image::ConvertBuffer;
    use std::fs::File;
    use std::ops::Deref;
    use approx::relative_eq;


    #[test]
    fn test_add_one_sample() {
        let crop_window = ((0.0, 0.0), (1.0, 1.0)).into();
        let filter = BoxFilter::default();
        let mut film = Film::new(Point2i::new(10, 10), crop_window, filter, 1.0);
        dbg!(&film);

        let tile_sample_bounds = ((0, 0), (2, 2)).into();
        let mut tile = film.get_film_tile(tile_sample_bounds);
        let sample = Spectrum::new(1.0);
        film.add_sample_to_tile(&mut tile, Point2f::new(1.0, 1.0), sample, 1.0);
        
        film.merge_film_tile(tile);


        let img = film.into_image_buffer();
        // TODO: approx assertions
//        let mut file = File::create("test.hdr").unwrap();
//        let encoder = image::hdr::HDREncoder::new(file);
//        let pixels: Vec<_> = img.pixels().map(|p| *p).collect();
//        encoder.encode(pixels.as_slice(), img.width() as usize, img.height() as usize).unwrap();
    }

}

