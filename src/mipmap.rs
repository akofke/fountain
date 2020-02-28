use crate::{Point2i, Float, Point2f, Lerp};
use crate::blocked_array::BlockedArray;
use crate::spectrum::Spectrum;

pub trait Texel: Copy + Clone + Sized + Default + std::ops::Mul<Float, Output=Self> + From<Float> + std::ops::AddAssign + std::ops::Add<Output=Self> + Lerp
{}

impl Texel for Float {}

impl Texel for Spectrum {}

#[derive(Clone, Copy)]
pub enum ImageWrap {
    Repeat, Black, Clamp,
}

pub struct MIPMap<T: Texel> {
    wrap_mode: ImageWrap,
    resolution: (usize, usize),
    pyramid: Vec<BlockedArray<T, 2>>,
}

struct ResampleWeight {
    first_texel: usize,
    weights: [Float; 4],
}

fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1) == 0)
}

fn log2int(n: usize) -> usize {
    (31 - n.leading_zeros()) as usize
}

///
/// ```
/// # use raytracer::mipmap::ceil_pow2;
/// assert_eq!(ceil_pow2(1), 1);
/// assert_eq!(ceil_pow2(3), 4);
/// assert_eq!(ceil_pow2(4), 4);
/// assert_eq!(ceil_pow2(13), 16);
/// ```
pub fn ceil_pow2(n: usize) -> usize {
    if is_power_of_two(n) { return n };
    let log2 = 31 - n.leading_zeros();
    2usize.pow(log2 + 1)
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
        x * lanczos
    }
}

impl<T: Texel> MIPMap<T> {

    pub fn new(
        resolution: (usize, usize),
        image: Vec<T>,
        wrap_mode: ImageWrap
    ) -> Self {
        let (image, resolution) = if !is_power_of_two(resolution.0) || !is_power_of_two(resolution.1) {
            let res_pow2 = (ceil_pow2(resolution.0), ceil_pow2(resolution.1));
            // resample to power of 2 res
            let s_weights = Self::resample_weights(resolution.0 as usize, res_pow2.0 as usize);

            let mut resampled_image = vec![T::from(0.0); (res_pow2.0 * res_pow2.1) as usize];

            // loop over every row in the original image
            for t in 0..resolution.1 {
                for s in 0..(res_pow2.0 as usize) {
                    let weight = &s_weights[s];
                    for (orig_s, wt) in (weight.first_texel .. weight.first_texel + 4).zip(&weight.weights) {
                        let orig_s = match wrap_mode {
                            ImageWrap::Repeat => orig_s % resolution.0 as usize,
                            ImageWrap::Black => orig_s,
                            ImageWrap::Clamp => orig_s.clamp(0, (resolution.0 - 1) as usize),
                        };

                        if orig_s < resolution.0 as usize{
                            resampled_image[t * res_pow2.0 + s] += image[t * resolution.0 + orig_s] * *wt;
                        }
                    }
                }
            }

            // TODO deduplicate
            let t_weights = Self::resample_weights(resolution.1 as usize, res_pow2.1 as usize);
            for s in 0..resolution.0 {
                for t in 0..res_pow2.1 {
                    let weight = &t_weights[t];
                    for (orig_t, wt) in (weight.first_texel .. weight.first_texel + 4).zip(&weight.weights) {
                        let orig_s = match wrap_mode {
                            ImageWrap::Repeat => orig_t % resolution.1 as usize,
                            ImageWrap::Black => orig_t,
                            ImageWrap::Clamp => orig_t.clamp(0, (resolution.1 - 1) as usize),
                        };

                        if orig_s < resolution.1 as usize{
                            resampled_image[t * res_pow2.1 + s] += image[t * resolution.0 + orig_s] * *wt;
                        }
                    }
                }
            }
            (resampled_image, res_pow2)
        } else {
            (image, resolution)
        };

        let n_levels = 1 + log2int(usize::max(resolution.0, resolution.1));

        let bottom_level = BlockedArray::with_default_block_size(&image, resolution.0, resolution.1);
        let mut pyramid = vec![bottom_level];

        (1..n_levels)
            .fold((resolution.0, resolution.1), |(s_res, t_res), _| {
                let s_res = usize::max(1, s_res / 2);
                let t_res = usize::max(1, t_res / 2);
                let mut level: BlockedArray<T, 2> = BlockedArray::default(s_res, t_res);
                let prev_level = pyramid.last().unwrap();

                for t in 0..t_res {
                    for s in 0..s_res {
                        let texel_sum =
                            Self::get_texel_from_level(prev_level, s*2, t*2, wrap_mode)
                                + Self::get_texel_from_level(prev_level, s*2 + 1, t*2, wrap_mode)
                                + Self::get_texel_from_level(prev_level, s*2, t*2 + 1, wrap_mode)
                                + Self::get_texel_from_level(prev_level, s*2 + 1, t*2 + 1, wrap_mode);
                        let filtered_texel = texel_sum * 0.25;
                        level[(s, t)] = filtered_texel;
                    }
                }
                pyramid.push(level);
                (s_res, t_res)
            });

        Self {
            wrap_mode,
            resolution,
            pyramid,
        }
    }

    pub fn lookup_trilinear(&self, st: Point2f, width: Float) -> T {
        // find the (continuous) level of the pyramid where the texels have a spacing of `width`
        let level = self.levels() as Float - 1.0 + (Float::max(width, 1.0e-8)).log2();

        if level < 0.0 {
            self.triangle(0, st)
        } else if level >= (self.levels() - 1) as Float {
            self.texel(self.levels() - 1, 0, 0)
        } else {
            let level_floor = level.floor() as usize;
            let delta = level.fract();
            T::lerp(delta, self.triangle(level_floor, st), self.triangle(level_floor, st))
        }
    }

    /// Filter four texels at a certain mipmap level around a given continuous texel coordinate
    fn triangle(&self, level: usize, st: Point2f) -> T {
        let level = level.clamp(0, self.levels() - 1);
        let level_array = &self.pyramid[level];
        let s = st.x * level_array.u_size() as Float - 0.5;
        let t = st.y * level_array.v_size() as Float - 0.5;
        let s0 = s.floor() as usize;
        let t0 = t.floor() as usize;
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

    fn texel(&self, level: usize, s: usize, t: usize) -> T {
        Self::get_texel_from_level(&self.pyramid[level], s, t, self.wrap_mode)
    }

    fn get_texel_from_level(level: &BlockedArray<T, 2>, s: usize, t: usize, wrap_mode: ImageWrap) -> T {
        let (s_size, t_size) = level.dimensions();
        let (s, t) = match wrap_mode {
            ImageWrap::Repeat => (s % s_size, t % t_size),
            ImageWrap::Clamp => (s.clamp(0, s_size - 1), t.clamp(0, t_size - 1)),
            ImageWrap::Black => {
                if s >= s_size || t >= t_size {
                    return 0.0.into()
                } else {
                    (s, t)
                }
            },
        };
        level[(s, t)]
    }

    fn resample_weights(old_res: usize, new_res: usize) -> Vec<ResampleWeight> {
        assert!(new_res >= old_res);
        let filter_width = 2.0;

        (0..new_res).into_iter()
            .map(|i| {
                // find the continuous coordinates of the new texel in terms of the old texel coordinates
                let center = (i as Float + 0.5) * old_res as Float / new_res as Float;
                let first_texel = ((center - filter_width) + 0.5).floor() as usize;
                let mut weights = [0.0; 4];
                for j in 0..4 {
                    let pos = (first_texel + j) as Float + 0.5;
                    weights[j] = lanczos_sinc((pos - center) / filter_width, 2.0);
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