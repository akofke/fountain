use crate::{Point2i, Float, Point2f, Lerp, Vec2f};
use crate::blocked_array::BlockedArray;
use crate::spectrum::Spectrum;

pub trait Texel: Copy + Clone + Sized + Default + std::ops::Mul<Float, Output=Self> + From<Float> + std::ops::AddAssign + std::ops::Add<Output=Self> + Lerp
{}

impl Texel for Float {}

impl Texel for Spectrum {}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ImageWrap {
    Repeat, Black, Clamp,
}

pub struct MIPMap<T: Texel> {
    wrap_mode: ImageWrap,
    resolution: (usize, usize), 
    pyramid: Vec<BlockedArray<T, 2>>,
}

struct ResampleWeight {
    first_texel: i32,
    weights: [Float; 4],
}

fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1) == 0)
}

fn log2_usize(n: usize) -> usize {
    (63 - n.leading_zeros()) as usize
}

fn lanczos_sinc(x: Float, tau: Float) -> Float {
    let x = x.abs();
    if x > 1.0 {
        0.0
    } else if x < 1e-5 {
        1.0
    } else {
        let x = x * std::f32::consts::PI;
        let s = Float::sin(x * tau) / (x * tau);
        let lanczos = Float::sin(x) / x;
        s * lanczos
    }
}

impl<T: Texel> MIPMap<T> {

    pub fn new(
        resolution: (usize, usize),
        image: Vec<T>,
        wrap_mode: ImageWrap
    ) -> Self {
        let (image, resolution) = if !is_power_of_two(resolution.0) || !is_power_of_two(resolution.1) {
            let res_pow2 = (resolution.0.next_power_of_two(), resolution.1.next_power_of_two());
            // let resolution = (resolution.0 as i32, resolution.1 as i32);
            // resample to power of 2 res
            let s_weights = Self::resample_weights(resolution.0, res_pow2.0);

            let mut resampled_image = vec![T::from(0.0); (res_pow2.0 * res_pow2.1) as usize];

            // loop over every row in the original image
            for t in 0..resolution.1 {
                // for every column in the upscaled image
                for s in 0..res_pow2.0 {
                    let weight = &s_weights[s as usize];
                    for (orig_s, wt) in (weight.first_texel .. weight.first_texel + 4).zip(&weight.weights) {
                        let orig_s = match wrap_mode {
                            ImageWrap::Repeat => orig_s.rem_euclid(resolution.0 as i32),
                            ImageWrap::Black => orig_s,
                            ImageWrap::Clamp => orig_s.clamp(0, resolution.0 as i32 - 1),
                        };

                        if orig_s >= 0 && orig_s < resolution.0 as i32 {
                            resampled_image[t * res_pow2.0 + s] += image[t * resolution.0 + orig_s as usize] * *wt;
                        }
                    }
                }
            }

            // TODO deduplicate
            let t_weights = Self::resample_weights(resolution.1 as usize, res_pow2.1 as usize);
            for s in 0..res_pow2.0 {
                for t in 0..res_pow2.1 {
                    let weight = &t_weights[t as usize];
                    let mut weighted_value = T::from(0.0);
                    for (orig_t, wt) in (weight.first_texel .. weight.first_texel + 4).zip(&weight.weights) {
                        let orig_t = match wrap_mode {
                            ImageWrap::Repeat => orig_t.rem_euclid(resolution.1 as i32),
                            ImageWrap::Black => orig_t,
                            ImageWrap::Clamp => orig_t.clamp(0, resolution.1 as i32 - 1),
                        };

                        if orig_t >= 0 && orig_t < resolution.1 as i32 {
                            let orig_t = orig_t as usize;
                            weighted_value += resampled_image[orig_t * res_pow2.0 + s] * *wt;
                        }
                    }
                    resampled_image[t * res_pow2.0 + s] = weighted_value;
                }
            }
            (resampled_image, res_pow2)
        } else {
            (image, resolution)
        };

        let n_levels = 1 + log2_usize(usize::max(resolution.0 as usize, resolution.1 as usize));

        let bottom_level = BlockedArray::with_default_block_size(&image, resolution.0 as usize, resolution.1 as usize);
        let mut pyramid = vec![bottom_level];

        (1..n_levels)
            .fold((resolution.0, resolution.1), |(s_res, t_res), _| {
                let s_res = usize::max(1, s_res / 2);
                let t_res = usize::max(1, t_res / 2);
                let mut level: BlockedArray<T, 2> = BlockedArray::default(s_res, t_res);
                let prev_level = pyramid.last().unwrap();

                for t in 0..t_res as i32 {
                    for s in 0..s_res as i32 {
                        let texel_sum =
                            Self::get_texel_from_level(prev_level, s*2, t*2, wrap_mode)
                                + Self::get_texel_from_level(prev_level, s*2 + 1, t*2, wrap_mode)
                                + Self::get_texel_from_level(prev_level, s*2, t*2 + 1, wrap_mode)
                                + Self::get_texel_from_level(prev_level, s*2 + 1, t*2 + 1, wrap_mode);
                        let filtered_texel = texel_sum * 0.25;
                        level[(s as usize, t as usize)] = filtered_texel;
                    }
                }
                pyramid.push(level);
                (s_res, t_res)
            });

        let resolution = (resolution.0 as usize, resolution.1 as usize);
        Self {
            wrap_mode,
            resolution,
            pyramid,
        }
    }

    pub fn lookup_trilinear_width(&self, st: Point2f, width: Float) -> T {
        // find the (continuous) level of the pyramid where the texels have a spacing of `width`
        let level = self.levels() as Float - 1.0 + (Float::max(width, 1.0e-8)).log2();
        if level < 0.0 {
            self.triangle(0, st)
        } else if level >= (self.levels() - 1) as Float {
            self.texel(self.levels() - 1, 0, 0)
        } else {
            let level_floor = level.floor() as usize;
            let delta = level.fract();
            T::lerp(delta, self.triangle(level_floor, st), self.triangle(level_floor + 1, st))
        }
    }

    pub fn lookup_trilinear(&self, st: Point2f, dst0: Vec2f, dst1: Vec2f) -> T {
        let width = (dst0.x.abs().max(dst0.y)).max(dst1.x.abs().max(dst1.y.abs()));
        self.lookup_trilinear_width(st, 2.0 * width)
    }

    /// Filter four texels at a certain mipmap level around a given continuous texel coordinate
    fn triangle(&self, level: usize, st: Point2f) -> T {
        let level = level.clamp(0, self.levels() - 1);
        let level_array = &self.pyramid[level];
        let s = st.x * level_array.u_size() as Float - 0.5;
        let t = st.y * level_array.v_size() as Float - 0.5;
        let s0 = s.floor() as i32;
        let t0 = t.floor() as i32;
        let ds = s - s0 as Float;
        let dt = t - t0 as Float;
        Self::get_texel_from_level(level_array, s0, t0, self.wrap_mode) * (1.0 - ds) * (1.0 - dt)
            + Self::get_texel_from_level(level_array, s0, t0 + 1, self.wrap_mode) * (1.0 - ds) * dt
            + Self::get_texel_from_level(level_array, s0 + 1, t0, self.wrap_mode) * ds * (1.0 - dt)
            + Self::get_texel_from_level(level_array, s0 + 1, t0 + 1, self.wrap_mode) * ds * dt

    }

    pub fn levels(&self) -> usize {
        self.pyramid.len()
    }

    pub fn pyramid(&self) -> &[BlockedArray<T, 2>] {
        &self.pyramid
    }

    fn texel(&self, level: usize, s: i32, t: i32) -> T {
        Self::get_texel_from_level(&self.pyramid[level], s, t, self.wrap_mode)
    }

    fn get_texel_from_level(level: &BlockedArray<T, 2>, s: i32, t: i32, wrap_mode: ImageWrap) -> T {
        let (s_size, t_size) = level.dimensions();
        let (s_size, t_size) = (s_size as i32, t_size as i32);
        let (s, t) = match wrap_mode {
            ImageWrap::Repeat => (s.rem_euclid(s_size), t.rem_euclid(t_size)),
            ImageWrap::Clamp => (s.clamp(0, s_size - 1), t.clamp(0, t_size - 1)),
            ImageWrap::Black => {
                if s < 0 || s >= s_size || t < 0 || t >= t_size {
                    return 0.0.into()
                } else {
                    (s, t)
                }
            },
        };
        level[(s as usize, t as usize)]
    }

    fn resample_weights(old_res: usize, new_res: usize) -> Vec<ResampleWeight> {
        assert!(new_res >= old_res);
        let filter_width = 2.0;

        (0..new_res).into_iter()
            .map(|i| {
                // find the continuous coordinates of the new texel in terms of the old texel coordinates
                let center = (i as Float + 0.5) * old_res as Float / new_res as Float;
                let first_texel = ((center - filter_width) + 0.5).floor() as i32;
                let mut weights = [0.0; 4];
                for j in 0..4 {
                    let pos = (first_texel + j) as Float + 0.5;
                    weights[j as usize] = lanczos_sinc((pos - center) / filter_width, 2.0);
                }

                let inv_sum_weights = 1.0 / weights.iter().sum::<Float>();
                for wt in weights.iter_mut() {
                    *wt *= inv_sum_weights;
                }
                ResampleWeight {
                    first_texel,
                    weights
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::prelude::*;
    use approx::assert_ulps_eq;

    #[test]
    fn test_mipmap_creation() {
        let img = vec![Spectrum::from(0.5); 256];
        let dims = (16, 16);
        let mipmap = MIPMap::new(dims, img, ImageWrap::Repeat);
    }

    #[test]
    fn test_mipmap_creation_non_pow2() {
        let img = vec![Spectrum::from(0.5); 200];
        let dims = (20, 10);
        let mipmap = MIPMap::new(dims, img, ImageWrap::Repeat);
    }

    #[test]
    fn test_mipmap_lookup() {
        let val = 0.5;
        let dims = (16, 15);
        let img = vec![val; dims.0 * dims.1];
        let mipmap = MIPMap::new(dims, img, ImageWrap::Repeat);

        let widths = Array::logspace(10.0, -4.0, 0.0, 10);
        let coords = Array1::linspace(0.0, 1.0, 25);
        for s in &coords {
            for t in &coords {
                for width in &widths {
                    let st = Point2f::new(*s, *t);
                    let filt = mipmap.lookup_trilinear_width(st, *width);
                    assert_ulps_eq!(filt, val, max_ulps=2)
                }
            }
        }
    }
}