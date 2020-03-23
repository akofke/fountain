use std::path::Path;
use crate::spectrum::Spectrum;
use crate::Float;
use std::fs::File;
use openexr::{InputFile, FrameBufferMut};

pub fn read_exr(path: impl AsRef<Path>) -> anyhow::Result<(Vec<Spectrum>, (usize, usize))> {
    let mut file = File::open(path)?; // TODO: BufReader
    let mut input_file = InputFile::new(&mut file)?;
    let (width, height) = input_file.header().data_dimensions();
    let (ox, oy) = input_file.header().data_origin();
    let mut pixels = vec![[0.0 as Float; 3]; (width * height) as usize];

    {
        let mut fb = FrameBufferMut::new_with_origin(ox, oy, width, height);
        fb.insert_channels(
            &[("R", 0.0), ("G", 0.0), ("B", 0.0)],
            &mut pixels
        );
        input_file.read_pixels(&mut fb)?;
    }
    // TODO: could transmute without copying
    let pixels = pixels.into_iter()
        .map(|rgb| {
            Spectrum::from(rgb)
        })
        .collect();

    Ok((pixels, (width as usize, height as usize)))
}