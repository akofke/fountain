use crate::mipmap::{ImageWrap, MIPMap};
use crate::Float;
use std::sync::Arc;
use crate::spectrum::{Spectrum, CoefficientSpectrum};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use image::io::Reader;
use image::{DynamicImage, Pixel, GenericImageView};
use std::collections::hash_map::Entry;

#[derive(PartialEq, Eq, Hash)]
pub struct ImageTexInfo {
    pub filename: String, // should probably be PathBuf
    pub wrap_mode: ImageWrap,
    // FIXME: ugly workaround
    pub scale_float_bits: u32,
    pub gamma: bool,
}

impl ImageTexInfo {
    pub fn new(filename: String, wrap_mode: ImageWrap, scale: Float, gamma: bool) -> Self {
        let scale_float_bits = scale.to_bits();
        Self {
            filename,
            wrap_mode,
            scale_float_bits,
            gamma
        }
    }

    pub fn scale(&self) -> Float {
        Float::from_bits(self.scale_float_bits)
    }
}

pub fn get_mipmap(info: ImageTexInfo) -> anyhow::Result<Arc<MIPMap<Spectrum>>> {
    // Global cache of mipmaps that have been loaded.
    static MIPMAPS: Lazy<Mutex<HashMap<ImageTexInfo, Arc<MIPMap<Spectrum>>>>> = Lazy::new(|| {
        Mutex::new(HashMap::new())
    });

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

pub fn load_mipmap(info: &ImageTexInfo) -> anyhow::Result<MIPMap<Spectrum>> {
    let image = Reader::open(&info.filename)?
        .decode()?;
    let dims = image.dimensions();
    let mut image: Vec<Spectrum> = match image {
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

    image.iter_mut().for_each(|s| {
        *s = s.map(|x| inverse_gamma_correct(x)) * info.scale()
    });

    let mipmap = MIPMap::new(
        (dims.0 as usize, dims.1 as usize),
        image,
        info.wrap_mode
    );
    Ok(mipmap)
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
