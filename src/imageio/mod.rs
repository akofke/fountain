use crate::mipmap::{ImageWrap, MIPMap};
use crate::Float;
use std::sync::Arc;
use crate::spectrum::{Spectrum};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use image::io::Reader;
use image::{DynamicImage, Pixel, GenericImageView, Rgb};
use std::collections::hash_map::Entry;
use core::iter;
use arrayvec::ArrayVec;
use crate::imageio::exr::read_exr;
use std::fmt::{Formatter, Debug};
use std::time::Instant;

pub mod exr;

#[derive(PartialEq, Eq, Hash)]
pub struct ImageTexInfo {
    pub filename: PathBuf,
    pub wrap_mode: ImageWrap,
    // FIXME: ugly workaround
    pub scale_float_bits: u32,
    pub gamma: Option<bool>,
    pub flip_y: bool,
}

impl ImageTexInfo {
    pub fn new(filename: impl Into<PathBuf>, wrap_mode: ImageWrap, scale: Float, gamma: Option<bool>, flip_y: bool) -> Self {
        let scale_float_bits = scale.to_bits();
        Self {
            filename: filename.into(),
            wrap_mode,
            scale_float_bits,
            gamma,
            flip_y
        }
    }

    pub fn scale(&self) -> Float {
        Float::from_bits(self.scale_float_bits)
    }
}

impl Debug for ImageTexInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageTexInfo")
            .field("filename", &self.filename)
            .field("wrap_mode", &self.wrap_mode)
            .field("scale", &f32::from_bits(self.scale_float_bits))
            .field("gamma", &self.gamma)
            .field("flip_y", &self.flip_y)
            .finish()
    }
}

#[tracing::instrument(skip(info))]
pub fn get_mipmap(info: ImageTexInfo) -> anyhow::Result<Arc<MIPMap<Spectrum>>> {
    // Global cache of mipmaps that have been loaded.
    static MIPMAPS: Lazy<Mutex<HashMap<ImageTexInfo, Arc<MIPMap<Spectrum>>>>> = Lazy::new(|| {
        Mutex::new(HashMap::new())
    });
    tracing::debug!(?info, "Requested mipmap");

    let mut cache = MIPMAPS.lock();
    match cache.entry(info) {
        Entry::Occupied(e) => {
            Ok(e.get().clone())
        },
        Entry::Vacant(e) => {
            let info = e.key();
            let mipmap = load_mipmap(info)?;
            Ok(e.insert(Arc::new(mipmap)).clone())
        },
    }
}

#[tracing::instrument(skip(info))]
pub fn load_mipmap(info: &ImageTexInfo) -> anyhow::Result<MIPMap<Spectrum>> {
    let start = Instant::now();
    let (mut image, dims) = load_image(&info.filename)?;

    // TODO: more robust handling of gamma correction/color spaces
    let gamma = match info.gamma {
        Some(g) => g,
        None => {
            if let Some(ext) = info.filename.extension() {
                match ext {
                    s if s == "exr" => false,
                    _ => true
                }
            } else {
                anyhow::bail!("No extension on image file {:?}", &info.filename)
            }
        }
    };

    image.iter_mut().for_each(|s| {
        *s = if gamma {
            s.map(inverse_gamma_correct)
        } else {
            *s
        } * info.scale()
    });

    if info.flip_y {
        for y in 0..dims.1 / 2 {
            for x in 0..dims.0 {
                let idx1 = y * dims.0 + x;
                let idx2 = (dims.1 - 1 - y) * dims.0 + x;
                image.swap(idx1, idx2);
            }
        }
    }

    let mipmap = MIPMap::new(
        (dims.0 as usize, dims.1 as usize),
        image,
        info.wrap_mode
    );
    tracing::debug!(time = ?start.elapsed().as_millis(), gamma, scale = ?info.scale());
    Ok(mipmap)
}

pub fn load_image(path: impl AsRef<Path>) -> anyhow::Result<(Vec<Spectrum>, (usize, usize))> {
    if let Some(ext) = path.as_ref().extension() {
        if ext == "exr" {
            return read_exr(path);
        }
    }
    let image = Reader::open(path)?.decode()?;
    let dims = image.dimensions();
    let image: Vec<Spectrum> = match image {
        DynamicImage::ImageRgb8(img) => {
            img.pixels().map(|p| {
                Spectrum::from_rgb8(p.to_rgb().0)
            }).collect()
        },
        DynamicImage::ImageRgba8(img) => {
            img.pixels().map(|p| {
                Spectrum::from_rgb8(p.to_rgb().0)
            }).collect()
        },
        _ => unimplemented!()
    };
    Ok((image, (dims.0 as usize, dims.1 as usize)))
}

pub fn spectrum_to_image(img: &[Spectrum], (w, h): (usize, usize)) -> image::RgbImage {
    let rgb_buf: Vec<u8> = img.iter()
        .flat_map(|s| {
            let rgb = s.map(gamma_correct).to_rgb8();
            ArrayVec::from(rgb).into_iter() // TODO
        })
        .collect();
    image::RgbImage::from_raw(w as u32, h as u32, rgb_buf).unwrap()
}

pub fn gamma_correct(v: Float) -> Float {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

pub fn inverse_gamma_correct(v: Float) -> Float {
    if v <= 0.04045 {
        v * 1.0 / 12.92
    } else {
        ((v + 0.055) * 1.0 / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}
